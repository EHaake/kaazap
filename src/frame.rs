use crate::{NUM_COLS, NUM_ROWS};

// Type alias for the Frame type
// Vector of vectors of borrowed static string slices
pub type Frame = Vec<Vec<&'static str>>;

pub fn new_frame() -> Frame {
    let mut cols = Vec::with_capacity(NUM_COLS);

    for _ in 0..NUM_COLS {
        // Create vector of NUM_ROWS amount of spaces " "
        let col = vec![" "; NUM_ROWS];
        cols.push(col);
    }

    cols
}

pub trait Drawable {
    fn draw(&self, frame: &mut Frame);
}
