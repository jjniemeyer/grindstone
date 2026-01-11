use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use std::time::Duration;

/// Application events
pub enum AppEvent {
    /// A key was pressed
    Key(KeyEvent),
    /// A tick occurred (for updating the timer display)
    Tick,
}

/// Poll for events with a timeout.
///
/// This function allows the timer to update while waiting for input.
/// Returns `Some(AppEvent)` if an event occurred, or `None` if no relevant event.
pub fn poll_event(tick_rate: Duration) -> color_eyre::Result<Option<AppEvent>> {
    if event::poll(tick_rate)? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => Ok(Some(AppEvent::Key(key))),
            _ => Ok(None),
        }
    } else {
        // No event within tick_rate, emit a tick for timer updates
        Ok(Some(AppEvent::Tick))
    }
}
