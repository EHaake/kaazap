use crate::config::Config;

// Type alias for the Frame type
// Vector of vectors of borrowed static string slices
pub type Frame = Vec<Vec<char>>;

pub fn new_frame(config: &Config) -> Frame {
    let mut cols = Vec::with_capacity(config.num_cols);

    for _ in 0..config.num_cols {
        // Create vector of num_rows amount of spaces " "
        // This will be the dynamic height of the terminal
        let col = vec![' '; config.num_rows];
        cols.push(col);
    }

    cols
}

pub trait Drawable {
    fn draw(&self, frame: &mut Frame);
}
