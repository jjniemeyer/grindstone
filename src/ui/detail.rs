use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;
use crate::models::Category;

/// Render the session detail modal as an overlay
pub fn render_detail_modal(frame: &mut Frame, area: Rect, app: &App) {
    // Calculate modal size and position (centered)
    let modal_width = 60.min(area.width.saturating_sub(4));
    let modal_height = 16.min(area.height.saturating_sub(4));
    let modal_x = (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

    // Clear the area behind the modal
    frame.render_widget(Clear, modal_area);

    // Modal block
    let block = Block::default()
        .title(" Session Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    // Get the selected session
    let session_idx = app.detail.selected_session_index;
    if session_idx >= app.data.sessions.len() {
        // Invalid index, show error
        let error_text = "Error: Session not found";
        frame.render_widget(
            Paragraph::new(error_text).centered().fg(Color::Red),
            inner,
        );
        return;
    }

    let session = &app.data.sessions[session_idx];

    let chunks = Layout::vertical([
        Constraint::Length(2), // Name
        Constraint::Length(2), // Category
        Constraint::Length(4), // Description
        Constraint::Length(2), // Duration
        Constraint::Length(2), // Started
        Constraint::Length(2), // Ended
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Controls
    ])
    .split(inner);

    // Name
    let name_line = Line::from(vec![
        Span::styled("Name: ", Style::default().bold()),
        Span::raw(&session.name),
    ]);
    frame.render_widget(Paragraph::new(name_line), chunks[0]);

    // Category with color
    let cat_color = get_category_color(&app.data.categories, &session.category);
    let category_line = Line::from(vec![
        Span::styled("Category: ", Style::default().bold()),
        Span::styled("■ ", Style::default().fg(cat_color)),
        Span::raw(&session.category),
    ]);
    frame.render_widget(Paragraph::new(category_line), chunks[1]);

    // Description
    let desc_text = if let Some(desc) = &session.description {
        desc.clone()
    } else {
        "(no description)".to_string()
    };
    let desc_style = if session.description.is_none() {
        Style::default().dark_gray()
    } else {
        Style::default()
    };
    let desc_line = vec![
        Line::from(vec![Span::styled("Description:", Style::default().bold())]),
        Line::from(vec![Span::styled(desc_text, desc_style)]),
    ];
    frame.render_widget(Paragraph::new(desc_line), chunks[2]);

    // Duration
    let duration_line = Line::from(vec![
        Span::styled("Duration: ", Style::default().bold()),
        Span::styled(
            session.format_duration(),
            Style::default().fg(Color::Yellow),
        ),
    ]);
    frame.render_widget(Paragraph::new(duration_line), chunks[3]);

    // Start time
    let start_dt = session.start_datetime();
    let start_str = start_dt.format("%H:%M on %A, %B %d, %Y").to_string();
    let start_line = Line::from(vec![
        Span::styled("Started: ", Style::default().bold()),
        Span::raw(start_str),
    ]);
    frame.render_widget(Paragraph::new(start_line), chunks[4]);

    // End time
    let end_dt = session.end_datetime();
    let end_str = end_dt.format("%H:%M on %A, %B %d, %Y").to_string();
    let end_line = Line::from(vec![
        Span::styled("Ended: ", Style::default().bold()),
        Span::raw(end_str),
    ]);
    frame.render_widget(Paragraph::new(end_line), chunks[5]);

    // Controls
    let controls = Line::from(vec![
        Span::styled("[Esc]", Style::default().bold()),
        Span::raw(" Close"),
    ]);
    frame.render_widget(Paragraph::new(controls).centered().dark_gray(), chunks[7]);
}

/// Look up a category's color by name, with gray fallback
fn get_category_color(categories: &[Category], name: &str) -> Color {
    categories
        .iter()
        .find(|c| c.name == name)
        .map(|c| c.color)
        .unwrap_or(Color::Gray)
}
