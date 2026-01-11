use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::App;
use crate::timer::TimerPhase;

/// Render the timer view
pub fn render_timer(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Title
        Constraint::Length(5), // Timer display
        Constraint::Length(3), // Phase label
        Constraint::Length(3), // Session info
        Constraint::Min(1),    // Spacer
        Constraint::Length(3), // Controls
        Constraint::Length(3), // Stats bar
    ])
    .split(area);

    // Title
    let title = Line::from("Grindstone").bold().blue().centered();
    frame.render_widget(
        Paragraph::new(title).block(Block::default().borders(Borders::BOTTOM)),
        chunks[0],
    );

    // Timer display (big countdown)
    let remaining = app.timer.remaining();
    let minutes = remaining.as_secs() / 60;
    let seconds = remaining.as_secs() % 60;
    let time_str = format!("{:02}:{:02}", minutes, seconds);

    let timer_color = match app.timer.phase {
        TimerPhase::Work => Color::Red,
        TimerPhase::ShortBreak => Color::Green,
        TimerPhase::LongBreak => Color::Blue,
    };

    let timer_display = Paragraph::new(Line::from(vec![Span::styled(
        time_str,
        Style::default().fg(timer_color).bold(),
    )]))
    .centered()
    .block(Block::default());
    frame.render_widget(timer_display, chunks[1]);

    // Progress bar
    let progress = app.timer.progress();
    let gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(timer_color))
        .ratio(progress);
    frame.render_widget(gauge, chunks[2]);

    // Phase label
    let phase_text = app.timer.phase.label();
    let status = if app.timer.is_running() {
        ""
    } else if app.timer.is_paused() {
        " (PAUSED)"
    } else {
        " (READY)"
    };
    let phase_line = Line::from(vec![
        Span::styled(phase_text, Style::default().fg(timer_color).bold()),
        Span::raw(status),
    ]);
    frame.render_widget(Paragraph::new(phase_line).centered(), chunks[3]);

    // Session info
    let session_info = if let Some(ref session) = app.current_session {
        format!("Session: \"{}\" ({})", session.name, session.category)
    } else {
        "No session - press [n] to start a new session".to_string()
    };
    frame.render_widget(
        Paragraph::new(session_info).centered().dark_gray(),
        chunks[4],
    );

    // Controls
    let controls = if app.timer.phase.is_break() {
        "[s] Skip Break  [p] Pause  [r] Reset  [n] New Session"
    } else if app.timer.is_running() {
        "[p] Pause  [r] Reset"
    } else if app.timer.is_paused() {
        "[s] Resume  [r] Reset"
    } else {
        "[s] Start  [n] New Session"
    };
    let controls_line = Line::from(controls).centered().dark_gray();
    frame.render_widget(
        Paragraph::new(controls_line).block(Block::default().borders(Borders::TOP)),
        chunks[5],
    );

    // Navigation bar
    let nav = "[Tab] Timer  [h] History  [t] Stats  [q] Quit";
    frame.render_widget(Paragraph::new(nav).centered().dark_gray(), chunks[6]);
}
