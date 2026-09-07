pub(crate) use crossterm::terminal;

use crate::layout::{BOARD_BLOCK_HEIGHT, IN_MATCH_MIN_WIDTH};

#[derive(Debug, Copy, Clone)]
pub struct Config {
    pub num_cols: usize,
    pub num_rows: usize,
}

impl Config {
    /// The smallest terminal the layout supports, as (cols, rows). The board
    /// is a fixed-size block, but an always-visible in-match panel sits beside
    /// it, so the minimum is the board plus symmetric panel margins — 139 × 31.
    /// Wider/taller terminals center the board and pad the margins.
    pub fn min_size() -> (usize, usize) {
        (IN_MATCH_MIN_WIDTH, BOARD_BLOCK_HEIGHT)
    }

    /// Does a terminal of this size meet the minimum?
    pub fn fits(cols: usize, rows: usize) -> bool {
        let (min_cols, min_rows) = Self::min_size();
        cols >= min_cols && rows >= min_rows
    }

    // Return error so that program exits if terminal size is too small
    pub fn from_terminal() -> anyhow::Result<Self> {
        let (cols, rows) = terminal::size()?;
        let cols = cols as usize;
        let rows = rows as usize;

        if !Self::fits(cols, rows) {
            let (min_cols, min_rows) = Self::min_size();
            anyhow::bail!(
                "Your terminal is too small!\n\
                Minimum size required: {}x{}\n\
                Current size: {}x{}\n",
                min_cols,
                min_rows,
                cols,
                rows
            );
        }

        Ok(Self {
            num_cols: cols,
            num_rows: rows,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_fits_accepts_minimum_and_larger_rejects_smaller() {
        let (mc, mr) = Config::min_size();
        assert!(Config::fits(mc, mr)); // exactly the minimum is allowed
        assert!(Config::fits(mc + 50, mr + 20));
        assert!(!Config::fits(mc - 1, mr)); // one col short
        assert!(!Config::fits(mc, mr - 1)); // one row short
    }

    #[test]
    fn config_min_size_is_board_plus_panel_margins() {
        // The minimum terminal is the board plus symmetric panel margins —
        // one source of truth, no independent guess.
        assert_eq!(Config::min_size(), (IN_MATCH_MIN_WIDTH, BOARD_BLOCK_HEIGHT));
        assert_eq!(Config::min_size(), (139, 31)); // pins the concrete size
    }
}
