use chrono::{Datelike, Local};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::models::Category;
use crate::ui;

/// Render the history view
pub fn render_history(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Title
        Constraint::Min(1),    // Session list
        Constraint::Length(1), // Controls
        Constraint::Length(1), // Footer
    ])
    .split(area);

    // Title
    let title = Line::from("Session History").bold().blue().centered();
    frame.render_widget(
        Paragraph::new(title).block(Block::default().borders(Borders::BOTTOM)),
        chunks[0],
    );

    // Session list grouped by day
    let items: Vec<ListItem> = build_history_items(&app.data.sessions, &app.data.categories);

    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[1], &mut app.data.history_state);

    // Controls
    let controls = "[j/k] Navigate  [Enter] Details  [d] Delete";
    frame.render_widget(
        Paragraph::new(controls)
            .centered()
            .dark_gray()
            .block(Block::default().borders(Borders::TOP)),
        chunks[2],
    );

    // Footer / notification
    ui::render_footer(frame, chunks[3], app, "[Tab] Timer  [t] Stats  [q] Quit");
}

/// Look up a category's color by name, with gray fallback
fn get_category_color(categories: &[Category], name: &str) -> Color {
    categories
        .iter()
        .find(|c| c.name == name)
        .map(|c| c.color)
        .unwrap_or(Color::Gray)
}

fn build_history_items(
    sessions: &[crate::models::Session],
    categories: &[Category],
) -> Vec<ListItem<'static>> {
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
        let cat_color = get_category_color(categories, &session.category);

        let line = Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(session.name.clone(), Style::default().bold()),
            Span::raw("  "),
            Span::styled(session.category.clone(), Style::default().fg(cat_color)),
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
