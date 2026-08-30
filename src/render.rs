use crossterm::{QueueableCommand, cursor::MoveTo, style::{Attribute, Color, SetAttribute, SetBackgroundColor}, terminal::{Clear, ClearType}};

use crate::frame::{Emphasis, Frame};
use std::io::{Stdout, Write};

// Only render what changed between last_frame and curr_frame
// Have the option to force the rendering (only should need once such as at the beginning)
pub fn render(stdout: &mut Stdout, last_frame: &Frame, curr_frame: &Frame, force: bool) {
    if force {
        stdout.queue(SetBackgroundColor(Color::Grey)).unwrap();
        stdout.queue(Clear(ClearType::All)).unwrap();
        stdout.queue(SetBackgroundColor(Color::Black)).unwrap();
    }

    // Track the attribute currently active in the terminal so we only
    // emit a change when a drawn cell's emphasis differs from it.
    // SetAttribute persists across the cells we skip, so comparing
    // against the last *emitted* cell's emphasis is correct.
    let mut active = Emphasis::Normal;

    for (x, col) in curr_frame.iter().enumerate() {
        for (y, cell) in col.iter().enumerate() {
            // If the cell has changed or we're forcing,
            if *cell != last_frame[x][y] || force {
                // move to the correct location,
                stdout.queue(MoveTo(x as u16, y as u16)).unwrap();

                // switch attribute only when it actually differs
                if cell.emphasis != active {
                    stdout.queue(SetAttribute(Attribute::Reset)).unwrap();
                    if cell.emphasis != Emphasis::Normal {
                        stdout.queue(SetAttribute(cell.emphasis.attribute())).unwrap();
                    }
                    active = cell.emphasis;
                }

                // and print the single char at the location
                print!("{}", cell.ch);
            }
        }
    }

    // Need to flush at the end since we've queued a bunch of commands
    stdout.flush().unwrap();
}
