use crossterm::{QueueableCommand, cursor::MoveTo, style::{Attribute, SetAttribute}, terminal::{Clear, ClearType}};

use crate::frame::{Emphasis, Frame};
use std::io::{Stdout, Write};

// Only render what changed between last_frame and curr_frame
// Have the option to force the rendering (only should need once such as at the beginning)
pub fn render(stdout: &mut Stdout, last_frame: &Frame, curr_frame: &Frame, force: bool) {
    // Start every frame from a known attribute baseline: reset SGR so an
    // attribute left active at the end of the previous frame can't leak
    // into cells we redraw here. This only affects characters we draw
    // next — glyphs already on screen keep their appearance. No colors
    // are ever set, so cleared/default cells show the terminal's own
    // background (design/brief.md: default background).
    stdout.queue(SetAttribute(Attribute::Reset)).unwrap();
    let mut active = Emphasis::Normal;

    if force {
        stdout.queue(Clear(ClearType::All)).unwrap();
    }

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
