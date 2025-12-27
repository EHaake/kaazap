use crossterm::{
    ExecutableCommand,
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use kaazap::{
    GAME_LOOP_SLEEP_MS, app::App, config::Config, frame::{self, new_frame}, render
};
use std::{io, sync::mpsc, thread, time::Duration};
// use rusty_audio::Audio;

fn main() -> anyhow::Result<()> {
    // Setup Audio
    // let mut audio = Audio::new();
    // audio.add("startup", "startup.wav");
    // audio.play("startup");

    // Terminal Initialization
    let config = Config::from_terminal()?;
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?; // Hide cursor

    // Initialize new Game State and Board
    // let board = BoardView::new(config.clone());
    // let mut game_state = GameState::new();

    // Initialize app
    let mut app = App::new(config.clone());

    // Render Loop
    //
    // Use separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_config = config.clone();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame(&render_config);
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
    //
    'gameloop: loop {
        let mut curr_frame = new_frame(&config);

        // Input handling:
        //
        // Poll for input events with default input,
        // which returns immediately if nothing to act upon
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    // System commands
                    KeyCode::Esc | KeyCode::Char('q') => {
                        break 'gameloop;
                    }
                    // Game commands
                    KeyCode::Char(c) => {
                        // if let Some(action) = game_state.handle_input(c) {
                        //     game_state.apply_action(action);
                        // }
                        app.handle_key(c);
                    }
                    _ => {}
                }
            }
        }

        // Updates
        //
        // Update the game state, checking for new states
        // game_state.update();
        app.tick();

        // Draw and render section
        //
        // board.draw(&game_state, &mut curr_frame);
        app.draw(&mut curr_frame);

        //
        // Send the frame!
        // Ignore the result since the receiving end of the channel won't be ready for a while
        let _ = render_tx.send(curr_frame);
        // Sleep since our game loop is much faster than the render loop
        thread::sleep(Duration::from_millis(GAME_LOOP_SLEEP_MS));

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

    // audio.wait(); // wait for audio to finish so it isn't cut off
    stdout.execute(Show)?; // Re-show the cursor (since hidden in alternate screen)
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
