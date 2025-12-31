use crossterm::{
    ExecutableCommand,
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use kaazap::{
    GAME_LOOP_SLEEP_MS,
    app::App,
    config::Config,
    frame::{self, new_frame},
    render,
};
use std::{
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
// use rusty_audio::Audio;

fn main() -> anyhow::Result<()> {

    // TODO: Setup Audio
    // let mut audio = Audio::new();
    // audio.add("startup", "startup.wav");
    // audio.play("startup");

    // Terminal Initialization
    let config = Config::from_terminal()?;
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?; // Hide cursor

    // Initialize app
    let mut app = App::new(config.clone());

    // Initialize time for animations
    let mut last_frame_time = Instant::now();

    // Render Loop
    //
    // Use separate thread for rendering
    let (render_tx, render_rx) = mpsc::sync_channel(1);
    let render_config = config.clone();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame(&render_config);
        let mut stdout = io::stdout();
        // first frame so we need to force render and last frame is what we have
        render::render(&mut stdout, &last_frame, &last_frame, true);

        // incremental updates
        while let Ok(mut curr_frame) = render_rx.recv() {
            // Drain queued frames (only keep the most current)
            while let Ok(newer) = render_rx.try_recv() {
                curr_frame = newer;
            }
            // Now we're ready to render our frame
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // Game loop
    //
    'gameloop: loop {
        let mut curr_frame = new_frame(&config);

        // Input handling:
        //
        // Poll for input events with default input,
        // which returns immediately if nothing to act upon
        if event::poll(Duration::from_millis(0))?
            && let Event::Key(key_event) = event::read()?
        {
            match key_event.code {
                KeyCode::Char('q') => break 'gameloop,
                _ => app.handle_key(key_event.code),
            }
        }

        // Updates
        //
        // Update the game state, checking for new states
        let now = Instant::now();
        // Update time duration to send to app
        let dt = now.duration_since(last_frame_time);
        app.tick(dt);
        last_frame_time = now;

        // Draw and render section
        app.draw(&mut curr_frame);

        // Send the frame!
        // Ignore the result since the receiving end of the channel won't be ready for a while
        let _ = render_tx.try_send(curr_frame);
        // Sleep since our game loop is much faster than the render loop
        thread::sleep(Duration::from_millis(GAME_LOOP_SLEEP_MS));
    }

    // Cleanup and close
    //
    // First make sure threads are cleaned up
    drop(render_tx);
    render_handle.join().unwrap();

    // TODO: Cleanup audio once implemented
    // audio.wait(); // wait for audio to finish so it isn't cut off

    stdout.execute(Show)?; // Re-show the cursor (since hidden in alternate screen)
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
