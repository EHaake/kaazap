use crossterm::{QueueableCommand, cursor::MoveTo, style::{Color, SetBackgroundColor}, terminal::{Clear, ClearType}};

use crate::frame::Frame;
use std::io::{Stdout, Write};

// Only render what changed between last_frame and curr_frame
// Have the option to force the rendering (only should need once such as at the beginning)
pub fn render(stdout: &mut Stdout, last_frame: &Frame, curr_frame: &Frame, force: bool) {
    if force {
        stdout.queue(SetBackgroundColor(Color::Blue)).unwrap(); 
        stdout.queue(Clear(ClearType::All)).unwrap();
        stdout.queue(SetBackgroundColor(Color::Black)).unwrap(); 
    }

    for (x, col) in curr_frame.iter().enumerate() {
        for (y, s) in col.iter().enumerate() {
            // Now we have the x,y index and the actual char at our current frame's location

            // If the character has changed or we're forcing,
            if *s != last_frame[x][y] || force {
                // we'll queue up a command to move to the correct location
                stdout.queue(MoveTo(x as u16, y as u16)).unwrap();
                // and print single char at the location
                print!("{}", *s);
            }
        }
    }

    // Need to flush at the end since we've queued a bunch of commands
    stdout.flush().unwrap();
}
