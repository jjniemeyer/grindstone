use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, SettingsField};

/// Render the settings modal as an overlay
pub fn render_settings_modal(frame: &mut Frame, area: Rect, app: &App) {
    // Calculate modal size and position (centered)
    let modal_width = 45.min(area.width.saturating_sub(4));
    let modal_height = 14.min(area.height.saturating_sub(4));
    let modal_x = (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

    // Clear the area behind the modal
    frame.render_widget(Clear, modal_area);

    // Modal block
    let block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let chunks = Layout::vertical([
        Constraint::Length(2), // Work duration
        Constraint::Length(2), // Short break
        Constraint::Length(2), // Long break
        Constraint::Length(2), // Sessions until long break
        Constraint::Length(1), // Spacer
        Constraint::Length(2), // Controls
    ])
    .split(inner);

    // Helper to render a settings row
    let render_row = |field: SettingsField, label: &str, value: i64, unit: &str| {
        let is_selected = app.settings.field == field;
        let style = if is_selected {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default()
        };

        let value_text = if is_selected {
            format!("{}_", app.settings.editing_value)
        } else {
            format_duration_value(field, value)
        };

        Line::from(vec![
            Span::styled(format!("{:<24}", label), style),
            Span::styled(value_text, style),
            Span::raw(format!(" {}", unit)),
        ])
    };

    frame.render_widget(
        Paragraph::new(render_row(
            SettingsField::WorkDuration,
            "Work Duration:",
            app.settings.editing_config.work_duration_secs,
            "min",
        )),
        chunks[0],
    );

    frame.render_widget(
        Paragraph::new(render_row(
            SettingsField::ShortBreak,
            "Short Break:",
            app.settings.editing_config.short_break_secs,
            "min",
        )),
        chunks[1],
    );

    frame.render_widget(
        Paragraph::new(render_row(
            SettingsField::LongBreak,
            "Long Break:",
            app.settings.editing_config.long_break_secs,
            "min",
        )),
        chunks[2],
    );

    frame.render_widget(
        Paragraph::new(render_row(
            SettingsField::SessionsUntilLong,
            "Sessions until long break:",
            app.settings.editing_config.sessions_until_long_break,
            "",
        )),
        chunks[3],
    );

    // Controls
    let controls = Line::from(vec![
        Span::styled("[Enter]", Style::default().bold()),
        Span::raw(" Save   "),
        Span::styled("[Tab/↑↓]", Style::default().bold()),
        Span::raw(" Navigate   "),
        Span::styled("[Esc]", Style::default().bold()),
        Span::raw(" Cancel"),
    ]);
    frame.render_widget(Paragraph::new(controls).centered().dark_gray(), chunks[5]);
}

/// Format a config value for display (convert seconds to minutes for durations)
fn format_duration_value(field: SettingsField, value: i64) -> String {
    match field {
        SettingsField::WorkDuration | SettingsField::ShortBreak | SettingsField::LongBreak => {
            format!("{}", value / 60)
        }
        SettingsField::SessionsUntilLong => format!("{}", value),
    }
}
