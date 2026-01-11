use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, InputField};

/// Render the session input modal as an overlay
pub fn render_input_modal(frame: &mut Frame, area: Rect, app: &App) {
    // Calculate modal size and position (centered)
    let modal_width = 50.min(area.width.saturating_sub(4));
    let modal_height = 12.min(area.height.saturating_sub(4));
    let modal_x = (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

    // Clear the area behind the modal
    frame.render_widget(Clear, modal_area);

    // Modal block
    let block = Block::default()
        .title(" New Session ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let chunks = Layout::vertical([
        Constraint::Length(3), // Name field
        Constraint::Length(3), // Description field
        Constraint::Length(2), // Category selector
        Constraint::Length(2), // Controls
    ])
    .split(inner);

    // Name field
    let name_style = if app.input_field == InputField::Name {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let name_block = Block::default()
        .title("Name")
        .borders(Borders::ALL)
        .border_style(name_style);
    let name_text = if app.input_field == InputField::Name {
        format!("{}_", app.input_name)
    } else {
        app.input_name.clone()
    };
    frame.render_widget(Paragraph::new(name_text).block(name_block), chunks[0]);

    // Description field
    let desc_style = if app.input_field == InputField::Description {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let desc_block = Block::default()
        .title("Description (optional)")
        .borders(Borders::ALL)
        .border_style(desc_style);
    let desc_text = if app.input_field == InputField::Description {
        format!("{}_", app.input_description)
    } else {
        app.input_description.clone()
    };
    frame.render_widget(Paragraph::new(desc_text).block(desc_block), chunks[1]);

    // Category selector
    let cat_style = if app.input_field == InputField::Category {
        Style::default().fg(Color::Yellow).bold()
    } else {
        Style::default()
    };
    let category_line = Line::from(vec![
        Span::raw("Category: "),
        Span::styled("< ", Style::default().dark_gray()),
        Span::styled(&app.categories[app.selected_category].name, cat_style),
        Span::styled(" >", Style::default().dark_gray()),
        Span::raw("  (←/→ to change)"),
    ]);
    frame.render_widget(Paragraph::new(category_line).centered(), chunks[2]);

    // Controls
    let controls = Line::from(vec![
        Span::styled("[Enter]", Style::default().bold()),
        Span::raw(" Start   "),
        Span::styled("[Tab]", Style::default().bold()),
        Span::raw(" Next Field   "),
        Span::styled("[Esc]", Style::default().bold()),
        Span::raw(" Cancel"),
    ]);
    frame.render_widget(Paragraph::new(controls).centered().dark_gray(), chunks[3]);
}
