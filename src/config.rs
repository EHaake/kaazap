pub(crate) use crossterm::terminal;

use crate::layout::{BOARD_BLOCK_HEIGHT, BOARD_WIDTH};

#[derive(Debug, Copy, Clone)]
pub struct Config {
    pub num_cols: usize,
    pub num_rows: usize,
}

impl Config {
    /// The smallest terminal the layout supports, as (cols, rows): the fixed
    /// board block, 89 × 31. From WIDE_LAYOUT_MIN_WIDTH columns the match adds
    /// the opponent presence panel beside it (spec 026); wider/taller
    /// terminals center the board and pad the margins.
    pub fn min_size() -> (usize, usize) {
        (BOARD_WIDTH, BOARD_BLOCK_HEIGHT)
    }

    /// The two sizes every "fits" test measures: the 89×31 minimum (compact)
    /// and the 139×31 threshold (wide). Test-only.
    #[cfg(test)]
    pub fn fit_sizes() -> [Config; 2] {
        use crate::layout::WIDE_LAYOUT_MIN_WIDTH;
        let (cols, rows) = Self::min_size();
        [
            Config { num_cols: cols, num_rows: rows },
            Config { num_cols: WIDE_LAYOUT_MIN_WIDTH, num_rows: rows },
        ]
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
                Minimum size required: {} x {}\n\
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
    fn config_min_size_is_the_board_block() {
        // The minimum terminal is the fixed board block — one source of
        // truth, no independent guess. The fit-test sizes are that minimum
        // and the wide threshold.
        assert_eq!(Config::min_size(), (BOARD_WIDTH, BOARD_BLOCK_HEIGHT));
        assert_eq!(Config::min_size(), (89, 31)); // pins the concrete size
        let [compact, wide] = Config::fit_sizes();
        assert_eq!((compact.num_cols, compact.num_rows), (89, 31));
        assert_eq!((wide.num_cols, wide.num_rows), (139, 31));
    }
}
