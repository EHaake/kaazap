//! Lifetime and per-run statistics: match/round tallies per opponent per mode,
//! win/loss streaks, and campaign completions. Pure data plus logic — no
//! rendering and no engine dependency. Callers record completed matches at the
//! match-end seam and read derived figures (totals, win rates, combined
//! per-opponent records) off these structs.
//!
//! See `specs/020-stats-and-records`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Which mode a completed match counts toward.
#[derive(Clone, Copy)]
pub enum Mode {
    QuickPlay,
    Campaign,
}

/// Per-opponent match and round tallies, within a single mode.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct OpponentRecord {
    pub match_wins: u32,
    pub match_losses: u32,
    pub round_wins: u32,
    pub round_losses: u32,
}

/// A win/loss streak: how many in a row right now, and the best ever.
#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub struct Streak {
    pub current: u32,
    pub longest: u32,
}

impl Streak {
    /// Record a match outcome: a win extends the current streak (updating the
    /// longest if it's a new best); a loss resets the current streak to zero
    /// while preserving the longest.
    pub fn record(&mut self, won: bool) {
        if won {
            self.current += 1;
            self.longest = self.longest.max(self.current);
        } else {
            self.current = 0;
        }
    }
}

/// All per-opponent records for a single mode.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ModeRecord {
    #[serde(default)]
    opponents: BTreeMap<String, OpponentRecord>,
}

impl ModeRecord {
    /// The record for an opponent, or a default (all-zero) record when this
    /// mode has no entry for that opponent yet.
    pub fn get(&self, id: &str) -> OpponentRecord {
        self.opponents.get(id).cloned().unwrap_or_default()
    }

    /// A mutable handle to an opponent's record, creating a default one if
    /// absent.
    pub fn entry_mut(&mut self, id: &str) -> &mut OpponentRecord {
        self.opponents.entry(id.to_string()).or_default()
    }

    /// Match wins and match losses summed across every opponent in this mode.
    pub fn totals(&self) -> (u32, u32) {
        self.opponents.values().fold((0, 0), |(w, l), r| {
            (w + r.match_wins, l + r.match_losses)
        })
    }
}

/// Lifetime statistics across all play: per-mode records, the overall streak,
/// and the number of campaign completions.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct LifetimeStats {
    #[serde(default)]
    quick_play: ModeRecord,
    #[serde(default)]
    campaign: ModeRecord,
    #[serde(default)]
    overall_streak: Streak,
    #[serde(default)]
    campaign_completions: u32,
}

impl LifetimeStats {
    /// Record a completed match into lifetime stats: bump the mode's opponent
    /// record (a match win or loss, plus the round counts), then advance the
    /// overall streak.
    pub fn record_match(
        &mut self,
        mode: Mode,
        opponent_id: &str,
        player_won: bool,
        player_rounds: u32,
        opp_rounds: u32,
    ) {
        let mode_record = match mode {
            Mode::QuickPlay => &mut self.quick_play,
            Mode::Campaign => &mut self.campaign,
        };
        let record = mode_record.entry_mut(opponent_id);
        if player_won {
            record.match_wins += 1;
        } else {
            record.match_losses += 1;
        }
        record.round_wins += player_rounds;
        record.round_losses += opp_rounds;

        self.overall_streak.record(player_won);
    }

    /// Record that the player finished a full campaign run.
    pub fn record_campaign_completion(&mut self) {
        self.campaign_completions += 1;
    }

    /// The Quick Play mode record.
    pub fn quick_play(&self) -> &ModeRecord {
        &self.quick_play
    }

    /// The Campaign mode record.
    pub fn campaign(&self) -> &ModeRecord {
        &self.campaign
    }

    /// The overall (cross-mode) win/loss streak.
    pub fn overall_streak(&self) -> Streak {
        self.overall_streak
    }

    /// How many campaign runs the player has completed.
    pub fn campaign_completions(&self) -> u32 {
        self.campaign_completions
    }

    /// An opponent's combined record across both modes, summed field-by-field.
    pub fn combined_opponent(&self, id: &str) -> OpponentRecord {
        let q = self.quick_play.get(id);
        let c = self.campaign.get(id);
        OpponentRecord {
            match_wins: q.match_wins + c.match_wins,
            match_losses: q.match_losses + c.match_losses,
            round_wins: q.round_wins + c.round_wins,
            round_losses: q.round_losses + c.round_losses,
        }
    }
}

/// Flat statistics for a single run (session): overall match/round tallies and
/// the run's streak, not broken down by opponent.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct RunStats {
    #[serde(default)]
    pub match_wins: u32,
    #[serde(default)]
    pub match_losses: u32,
    #[serde(default)]
    pub round_wins: u32,
    #[serde(default)]
    pub round_losses: u32,
    #[serde(default)]
    pub streak: Streak,
}

impl RunStats {
    /// Record a completed match into the flat run tally, then advance the run
    /// streak.
    pub fn record_match(&mut self, player_won: bool, player_rounds: u32, opp_rounds: u32) {
        if player_won {
            self.match_wins += 1;
        } else {
            self.match_losses += 1;
        }
        self.round_wins += player_rounds;
        self.round_losses += opp_rounds;
        self.streak.record(player_won);
    }
}

/// The rounded integer win-rate percentage, or `None` when no matches have been
/// played (`wins + losses == 0`). Rounds to nearest rather than truncating, so
/// 2 of 3 is 67, not 66.
pub fn win_rate(wins: u32, losses: u32) -> Option<u32> {
    let total = wins + losses;
    if total == 0 {
        return None;
    }
    Some((wins * 100 + total / 2) / total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streak_record_raises_tracks_and_resets() {
        let mut s = Streak::default();
        s.record(true);
        s.record(true);
        s.record(true);
        assert_eq!(s.current, 3);
        assert_eq!(s.longest, 3);

        s.record(false);
        assert_eq!(s.current, 0);
        assert_eq!(s.longest, 3, "longest is preserved across a loss");

        s.record(true);
        assert_eq!(s.current, 1);
        assert_eq!(s.longest, 3, "longest unchanged until current exceeds it");
    }

    #[test]
    fn record_match_credits_correct_mode() {
        let mut stats = LifetimeStats::default();

        // A Quick Play win, 3 rounds to 1.
        stats.record_match(Mode::QuickPlay, "yuka", true, 3, 1);
        let q = stats.quick_play().get("yuka");
        assert_eq!(q.match_wins, 1);
        assert_eq!(q.match_losses, 0);
        assert_eq!(q.round_wins, 3);
        assert_eq!(q.round_losses, 1);
        // Campaign untouched.
        let c = stats.campaign().get("yuka");
        assert_eq!(c.match_wins, 0);
        assert_eq!(c.match_losses, 0);
        assert_eq!(c.round_wins, 0);
        assert_eq!(c.round_losses, 0);

        // A Campaign loss, 2 rounds to 3.
        stats.record_match(Mode::Campaign, "yuka", false, 2, 3);
        let c = stats.campaign().get("yuka");
        assert_eq!(c.match_wins, 0);
        assert_eq!(c.match_losses, 1);
        assert_eq!(c.round_wins, 2);
        assert_eq!(c.round_losses, 3);
        // Quick Play unchanged by the campaign match.
        let q = stats.quick_play().get("yuka");
        assert_eq!(q.match_wins, 1);
        assert_eq!(q.match_losses, 0);
    }

    #[test]
    fn combined_opponent_sums_both_modes() {
        let mut stats = LifetimeStats::default();
        stats.record_match(Mode::QuickPlay, "yuka", true, 3, 1);
        stats.record_match(Mode::Campaign, "yuka", false, 2, 3);

        let combined = stats.combined_opponent("yuka");
        assert_eq!(combined.match_wins, 1);
        assert_eq!(combined.match_losses, 1);
        assert_eq!(combined.round_wins, 5);
        assert_eq!(combined.round_losses, 4);
    }

    #[test]
    fn overall_streak_advances_and_resets() {
        let mut stats = LifetimeStats::default();
        // win, win, loss, win across modes.
        stats.record_match(Mode::QuickPlay, "a", true, 3, 0);
        stats.record_match(Mode::Campaign, "b", true, 3, 1);
        assert_eq!(stats.overall_streak().current, 2);
        assert_eq!(stats.overall_streak().longest, 2);

        stats.record_match(Mode::QuickPlay, "c", false, 1, 3);
        assert_eq!(stats.overall_streak().current, 0);
        assert_eq!(stats.overall_streak().longest, 2);

        stats.record_match(Mode::Campaign, "d", true, 3, 2);
        assert_eq!(stats.overall_streak().current, 1);
        assert_eq!(stats.overall_streak().longest, 2);
    }

    #[test]
    fn mode_record_totals_sum_across_opponents() {
        let mut stats = LifetimeStats::default();
        stats.record_match(Mode::QuickPlay, "a", true, 3, 1);
        stats.record_match(Mode::QuickPlay, "a", false, 2, 3);
        stats.record_match(Mode::QuickPlay, "b", true, 3, 0);

        let (wins, losses) = stats.quick_play().totals();
        assert_eq!(wins, 2);
        assert_eq!(losses, 1);
    }

    #[test]
    fn win_rate_none_at_zero_and_rounds_otherwise() {
        assert_eq!(win_rate(0, 0), None);
        assert_eq!(win_rate(3, 1), Some(75)); // 3 of 4
        assert_eq!(win_rate(2, 1), Some(67)); // 2 of 3, rounds up from 66.6
        assert_eq!(win_rate(1, 1), Some(50));
    }
}
