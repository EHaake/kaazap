pub(crate) use crossterm::terminal;

use crate::layout::{BOARD_BLOCK_HEIGHT, BOARD_WIDTH};

#[derive(Debug, Copy, Clone)]
pub struct Config {
    pub num_cols: usize,
    pub num_rows: usize,
}

impl Config {
    /// The smallest terminal the layout supports, as (cols, rows). The
    /// board is a fixed-size block, so the minimum terminal is exactly big
    /// enough to hold it — ~89 × 30, wider than a classic 80×24 terminal.
    /// Larger terminals center the same block and pad the margins.
    pub fn min_size() -> (usize, usize) {
        (BOARD_WIDTH, BOARD_BLOCK_HEIGHT)
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
    fn config_min_size_is_the_fixed_board_size() {
        // The minimum terminal is exactly the fixed board — one source of
        // truth, no independent guess.
        let (min_cols, min_rows) = Config::min_size();
        assert_eq!(min_cols, BOARD_WIDTH);
        assert_eq!(min_rows, BOARD_BLOCK_HEIGHT);
        assert_eq!((min_cols, min_rows), (89, 31)); // pins the concrete size
    }
}
