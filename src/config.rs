use crossterm::terminal;

#[derive(Debug, Clone)]
pub struct Config {
    pub num_cols: usize,
    pub num_rows: usize,
}

impl Config {
    pub fn from_terminal() -> anyhow::Result<Self> {
        let (cols, rows) = terminal::size()?;

        Ok(Self {
            num_cols: cols as usize,
            num_rows: rows as usize,
        })
    }
}
