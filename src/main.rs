use std::{error::Error, io, sync::mpsc, thread, time::{Duration, Instant}};

use crossterm::{
    ExecutableCommand, cursor::{Hide, Show}, event::{self, Event, KeyCode}, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}
};
use kaazap::{frame::{self, new_frame}, render};
use rusty_audio::Audio;

fn main() -> Result<(), Box<dyn Error>> {
    // Setup Audio
    let mut audio = Audio::new();
    // audio.add("startup", "startup.wav");
    // audio.play("startup");

    // Terminal Initialization
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?; // Hide cursor

    // Render Loop
    //
    // Use separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        // first frame so we need to force render and last frame is what we have
        render::render(&mut stdout, &last_frame, &last_frame, true);

        // incremental updates
        while let Ok(frame) = render_rx.recv() {
            let curr_frame = frame;
            // Now we're ready to render our frame
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });



    // Game loop
    //
    // Setup
    let mut instant = Instant::now(); 
    //
    'gameloop: loop {
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();


        // Input handling:
        //
        // Poll for input events with default input,
        // which returns immediately if nothing to act upon
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        // audio.play("lose");
                        break 'gameloop;
                    }
                    // TODO: Add the rest of the keymaps
                    _ => {}
                }
            }
        }

        // Updates
        //
        



        // Win or lose section
        //
        // TODO: Add win/lose conditions
        
        // if game.won() {
        //     audio.play("win");
        //     break 'gameloop;
        // }
        // if game.lost() {
        //     audio.play("lose");
        //     break 'gameloop;
        // }
    }








    // Cleanup and close
    //
    // First make sure threads are cleaned up
    drop(render_tx);
    render_handle.join().unwrap();

    audio.wait(); // wait for audio to finish so it isn't cut off
    stdout.execute(Show)?; // Re-show the cursor (since hidden in alternate screen)
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
