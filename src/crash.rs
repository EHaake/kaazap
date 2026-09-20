//! How kaazap ends (spec 028): the terminal restore, and the crash report.
//!
//! The panic hook here *records* and prints nothing — at the moment a panic
//! fires, the alternate screen is still up and anything printed on it is
//! discarded when we leave it. `main`'s terminal guard restores the terminal
//! first and prints the recorded report after, on the terminal the player
//! came from.

use std::{
    io,
    panic::{self, PanicHookInfo},
    sync::OnceLock,
};

use crossterm::{
    ExecutableCommand,
    cursor::Show,
    terminal::{self, LeaveAlternateScreen},
};

/// The first panic of this run. First wins: a panic on the render thread is
/// the cause, and the `join().unwrap()` that surfaces it on the main thread
/// at shutdown is only the symptom.
static REPORT: OnceLock<CrashReport> = OnceLock::new();

/// A recorded panic: what it said, and where it happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashReport {
    pub message: String,
    pub location: Option<String>,
}

/// Record every panic, print nothing. Installed by the terminal guard as its
/// first act, before raw mode — the guard exists by then, so anything recorded
/// is printed after the restore, and no window is left where a panic prints
/// onto the alternate screen and is thrown away with it.
pub fn install_hook() {
    panic::set_hook(Box::new(|info: &PanicHookInfo| {
        // First panic wins: a later `set` against an already-filled `OnceLock`
        // is a no-op, which is exactly what the shutdown `join().unwrap()`
        // should be.
        let _ = REPORT.set(CrashReport {
            message: info.payload_as_str().unwrap_or("(no message)").to_string(),
            location: info.location().map(|l| l.to_string()),
        });
    }));
}

/// The recorded panic, if this run crashed.
pub fn report() -> Option<&'static CrashReport> {
    REPORT.get()
}

/// Put the terminal back the way it was found: cursor shown, alternate screen
/// left, raw mode off. Every error is swallowed — this runs from a `Drop`
/// during unwinding, where a panic would abort the process, and running it
/// twice must change nothing.
pub fn restore_terminal() {
    let mut stdout = io::stdout();
    let _ = stdout.execute(Show);
    let _ = stdout.execute(LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}

/// What a crash prints on the restored terminal: one kaazap line, then the
/// panic's own message and location (spec Q5 a). Pure, so the wording is
/// tested without crashing anything.
pub fn crash_lines(report: &CrashReport) -> Vec<String> {
    let mut lines = vec![
        "kaazap crashed — this is a bug in the game, not something you did.".to_string(),
        format!("  {}", report.message),
    ];
    if let Some(at) = &report.location {
        lines.push(format!("  at {at}"));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    // `install_hook` is deliberately never called from a test: it replaces the
    // process-wide hook for the whole test binary, and a hook that prints
    // nothing would swallow the panic output of every failing test here.

    #[test]
    fn crash_lines_name_kaazap_then_the_message_and_location() {
        let report = CrashReport {
            message: "the dealer deck ran dry".to_string(),
            location: Some("src/game.rs:42:9".to_string()),
        };
        let lines = crash_lines(&report);
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("kaazap"), "first line: {}", lines[0]);
        assert!(
            lines[1].contains("the dealer deck ran dry"),
            "second line: {}",
            lines[1]
        );
        assert!(lines[2].starts_with("  at "), "third line: {}", lines[2]);
        assert!(lines[2].contains("src/game.rs:42:9"));

        let no_location = CrashReport {
            message: "the dealer deck ran dry".to_string(),
            location: None,
        };
        let lines = crash_lines(&no_location);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("kaazap"));
        assert!(lines[1].contains("the dealer deck ran dry"));
        assert!(!lines.iter().any(|line| line.starts_with("  at ")));
    }

    #[test]
    fn restoring_the_terminal_twice_is_harmless() {
        // No unwrap anywhere in it, so neither call can panic — the property
        // that keeps a `Drop` during unwinding from aborting the process.
        restore_terminal();
        restore_terminal();
    }
}
