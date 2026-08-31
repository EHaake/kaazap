pub(crate) use crossterm::terminal;

use crate::{CARD_WIDTH, HAND_SIZE, H_PAD};

#[derive(Debug, Copy, Clone)]
pub struct Config {
    pub num_cols: usize,
    pub num_rows: usize,
}

impl Config {
    /// The smallest terminal the layout supports, as (cols, rows).
    /// Width must fit a full HAND_SIZE-card hand on each side of the
    /// divider (~89 cols) — wider than a classic 80-column terminal.
    /// Height must fit the fixed board block at that (worst-case, tallest)
    /// width — the single source is `layout::board_block_height` (~30).
    pub fn min_size() -> (usize, usize) {
        let half = H_PAD + HAND_SIZE * (CARD_WIDTH + 1);
        let min_cols = 2 * half + 1;
        (min_cols, crate::layout::board_block_height(min_cols))
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
    fn config_min_height_matches_board_block_height_at_min_width() {
        // Single source of truth: the minimum height is exactly the board
        // block at the minimum (tallest) width, not an independent guess.
        let (min_cols, min_rows) = Config::min_size();
        assert_eq!(min_rows, crate::layout::board_block_height(min_cols));
        assert_eq!(min_rows, 30); // 3 grid rows + header/hand/status/gaps
    }
}
