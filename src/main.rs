use anyhow::Context;
use crossterm::{
    ExecutableCommand,
    cursor::Hide,
    event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen},
};
use kaazap::{
    GAME_LOOP_SLEEP_MS,
    app::{App, resolve_key},
    config::Config,
    crash,
    frame::{self, new_frame},
    render,
};
use std::{
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

fn main() -> anyhow::Result<()> {
    // Terminal Initialization
    let mut config = Config::from_terminal()?;

    // Declared first, so it is dropped *last*: locals drop in reverse order,
    // so the render thread's sender (and with it the render thread) is gone
    // before the terminal is restored and any crash report printed on it.
    let _terminal = TerminalGuard::enter()?;

    // The deliberate-crash seam (spec 028); `None` on every ordinary run.
    let crash_at = std::env::var("KAAZAP_CRASH_AT").ok();

    // Initialize app
    let mut app = App::new(config.clone());

    // Initialize time for animations
    let mut last_frame_time = Instant::now();

    // Render Loop
    //
    // Use separate thread for rendering
    let (render_tx, render_rx) = mpsc::sync_channel(1);
    let render_config = config.clone();
    let render_crash_at = crash_at.clone();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame(&render_config);
        let mut stdout = io::stdout();
        // first frame so we need to force render and last frame is what we have
        render::render(&mut stdout, &last_frame, &last_frame, true);

        // incremental updates
        while let Ok(mut curr_frame) = render_rx.recv() {
            // A crash is being reported on the real terminal — stop drawing.
            if crash::report().is_some() {
                break;
            }
            crash_if_requested(&render_crash_at, "render");
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
        if crash_at.as_deref() == Some("input") {
            // The `?` path below, forced: an input error leaves main through
            // a return, restoring the terminal on the way (spec 028 AC 4).
            anyhow::bail!("kaazap couldn't read the terminal (KAAZAP_CRASH_AT=input)");
        }

        // Input handling:
        //
        // Poll for input events with default input,
        // which returns immediately if nothing to act upon
        //
        // `.context` so the line the player sees names kaazap, like the crash
        // line does — an input error is not a panic and records no report.
        if event::poll(Duration::from_millis(0)).context("kaazap couldn't read the terminal")? {
            match event::read().context("kaazap couldn't read the terminal")? {
                Event::Key(key_event) => {
                    // Translate emacs nav chords (Ctrl+P/N/B/F) to arrows once,
                    // up front, so every screen sees them as the arrows they
                    // mirror.
                    let code = resolve_key(key_event.code, key_event.modifiers);
                    match code {
                        KeyCode::Char('q') => break 'gameloop,
                        _ => {
                            crash_if_requested(&crash_at, "key");
                            app.handle_key(code)
                        }
                    }
                }
                // Terminal resized: track the new size for frame
                // allocation, and either re-lay-out or show the
                // too-small screen (game state is preserved either way).
                Event::Resize(cols, rows) => {
                    let (cols, rows) = (cols as usize, rows as usize);
                    config = Config {
                        num_cols: cols,
                        num_rows: rows,
                    };
                    if Config::fits(cols, rows) {
                        app.resize(config);
                    } else {
                        app.set_too_small(cols, rows);
                    }
                }
                _ => {}
            }
        }

        // Allocate the frame at the current (possibly just-resized) size
        let mut curr_frame = new_frame(&config);

        // Updates
        //
        // Update the game state, checking for new states
        let now = Instant::now();
        // Update time duration to send to app
        let dt = now.duration_since(last_frame_time);
        crash_if_requested(&crash_at, "tick");
        app.tick(dt);
        last_frame_time = now;

        // Draw and render section
        crash_if_requested(&crash_at, "draw");
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
    render_handle.join().unwrap(); // a panicked render thread surfaces here

    Ok(())
}

/// Raw mode, the alternate screen and a hidden cursor — undone on every ending
/// spec 028 names: a clean quit, an error returned from `main`, and a panic on
/// either thread. `Drop` covers all three, so the restore has exactly one home
/// and the crash report is printed after it, on the real terminal. (Not a
/// signal: `kill` still leaves the terminal as it was.)
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> anyhow::Result<Self> {
        let guard = Self; // from here on, an early `?` restores too
        crash::install_hook(); // …and from here on, a panic is recorded and
        // printed by the Drop below, after the restore
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(Hide)?; // Hide cursor
        Ok(guard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        crash::restore_terminal();
        if let Some(report) = crash::report() {
            for line in crash::crash_lines(report) {
                eprintln!("{line}");
            }
        }
    }
}

/// Panic on purpose at `point`, if `KAAZAP_CRASH_AT` named it (spec 028).
/// Nothing in the game sets that variable and no key reaches this.
fn crash_if_requested(crash_at: &Option<String>, point: &str) {
    if crash_at.as_deref() == Some(point) {
        panic!("deliberate crash at {point} (KAAZAP_CRASH_AT)");
    }
}
