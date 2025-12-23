pub(crate) use crossterm::terminal;

use crate::{CARD_WIDTH, H_PAD, V_PAD};

#[derive(Debug, Clone)]
pub struct Config {
    pub num_cols: usize,
    pub num_rows: usize,
}

impl Config {
    // Return error so that program exits if terminal size is too small
    pub fn from_terminal() -> anyhow::Result<Self> {
        let (cols, rows) = terminal::size()?;

        let cols = cols as usize;
        let rows = rows as usize;

        let min_cols = CARD_WIDTH * 3 + H_PAD;
        let min_rows = CARD_WIDTH * 4 + V_PAD;

        if cols < min_cols || rows < min_rows {
            anyhow::bail!(
                "Your terminal is too small!\n\
                Minimum height required: {}x{}\n\
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
