use chrono::{Datelike, Local};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;

/// Render the history view
pub fn render_history(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Title
        Constraint::Min(1),    // Session list
        Constraint::Length(2), // Controls
    ])
    .split(area);

    // Title
    let title = Line::from("Session History").bold().blue().centered();
    frame.render_widget(
        Paragraph::new(title).block(Block::default().borders(Borders::BOTTOM)),
        chunks[0],
    );

    // Session list grouped by day
    let items: Vec<ListItem> = build_history_items(&app.sessions);

    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[1], &mut app.history_state);

    // Controls
    let controls = "[j/k] Navigate  [d] Delete  [Tab] Timer  [t] Stats  [q] Quit";
    frame.render_widget(
        Paragraph::new(controls)
            .centered()
            .dark_gray()
            .block(Block::default().borders(Borders::TOP)),
        chunks[2],
    );
}

fn build_history_items(sessions: &[crate::models::Session]) -> Vec<ListItem<'static>> {
    let mut items = Vec::new();
    let mut current_date: Option<(i32, u32, u32)> = None;
    let today = Local::now().date_naive();

    for session in sessions {
        let dt = session.start_datetime();
        let date = (dt.year(), dt.month(), dt.day());

        // Add date header if new day
        if current_date != Some(date) {
            current_date = Some(date);
            let date_str = if dt.date_naive() == today {
                "Today".to_string()
            } else if dt.date_naive() == today.pred_opt().unwrap_or(today) {
                "Yesterday".to_string()
            } else {
                dt.format("%A, %B %d, %Y").to_string()
            };
            items.push(ListItem::new(Line::from(vec![Span::styled(
                date_str,
                Style::default().bold().underlined(),
            )])));
        }

        // Session line
        let start_time = session.start_datetime().format("%H:%M");
        let end_time = session.end_datetime().format("%H:%M");
        let duration = session.format_duration();

        let line = Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(session.name.clone(), Style::default().bold()),
            Span::raw("  "),
            Span::styled(session.category.clone(), Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled(duration, Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled(
                format!("{} - {}", start_time, end_time),
                Style::default().dark_gray(),
            ),
        ]);
        items.push(ListItem::new(line));
    }

    if items.is_empty() {
        items.push(ListItem::new(
            Line::from("No sessions yet. Start a pomodoro!").centered(),
        ));
    }

    items
}
