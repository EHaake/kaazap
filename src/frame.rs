use crossterm::style::Attribute;

use crate::config::Config;

/// Monochrome emphasis — one axis at a time, per design/brief.md. Not
/// bitflags: representable state matches allowed state. Cursor
/// selection is carried by border-glyph weight, not by this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Emphasis {
    #[default]
    Normal, // default weight
    Strong, // bold    — the thing to look at
    Muted,  // dim     — inactive / secondary
    Alert,  // inverse — interrupts only, rationed
}

impl Emphasis {
    /// The terminal attribute this emphasis renders as. Normal maps to
    /// Reset (no attribute); only Emphasis ever becomes an escape code —
    /// there is no path here that emits color.
    pub fn attribute(self) -> Attribute {
        match self {
            Emphasis::Normal => Attribute::Reset,
            Emphasis::Strong => Attribute::Bold,
            Emphasis::Muted => Attribute::Dim,
            Emphasis::Alert => Attribute::Reverse,
        }
    }
}

/// One screen position: a character and its monochrome emphasis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub emphasis: Emphasis,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            emphasis: Emphasis::Normal,
        }
    }
}

// The double-buffered frame: a grid of styled cells, indexed [x][y].
pub type Frame = Vec<Vec<Cell>>;

pub fn new_frame(config: &Config) -> Frame {
    let mut cols = Vec::with_capacity(config.num_cols);

    for _ in 0..config.num_cols {
        // A fresh column of blank cells — the dynamic terminal height
        let col = vec![Cell::default(); config.num_rows];
        cols.push(col);
    }

    cols
}

pub trait Drawable {
    fn draw(&self, frame: &mut Frame);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_default_is_blank_and_normal() {
        let c = Cell::default();
        assert_eq!(c.ch, ' ');
        assert_eq!(c.emphasis, Emphasis::Normal);
    }

    #[test]
    fn emphasis_default_is_normal() {
        assert_eq!(Emphasis::default(), Emphasis::Normal);
    }

    #[test]
    fn emphasis_maps_to_the_expected_attributes() {
        assert_eq!(Emphasis::Normal.attribute(), Attribute::Reset);
        assert_eq!(Emphasis::Strong.attribute(), Attribute::Bold);
        assert_eq!(Emphasis::Muted.attribute(), Attribute::Dim);
        assert_eq!(Emphasis::Alert.attribute(), Attribute::Reverse);
    }
}
