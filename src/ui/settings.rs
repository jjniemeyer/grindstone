use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, CategoryField, SettingsField, SettingsMode};

/// Render the settings modal as an overlay
pub fn render_settings_modal(frame: &mut Frame, area: Rect, app: &App) {
    // Calculate modal size and position (centered)
    let modal_width = 50.min(area.width.saturating_sub(4));
    let modal_height = 18.min(area.height.saturating_sub(4));
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
        Constraint::Length(2), // Mode tabs
        Constraint::Min(1),    // Content area
        Constraint::Length(2), // Controls
    ])
    .split(inner);

    // Mode tabs
    render_mode_tabs(frame, chunks[0], app);

    // Content based on mode
    match app.settings.mode {
        SettingsMode::Timer => render_timer_settings(frame, chunks[1], app),
        SettingsMode::Categories => render_category_settings(frame, chunks[1], app),
    }

    // Controls based on mode
    let controls = match app.settings.mode {
        SettingsMode::Timer => Line::from(vec![
            Span::styled("[Enter]", Style::default().bold()),
            Span::raw(" Save  "),
            Span::styled("[Tab/↑↓]", Style::default().bold()),
            Span::raw(" Navigate  "),
            Span::styled("[1/2]", Style::default().bold()),
            Span::raw(" Mode  "),
            Span::styled("[Esc]", Style::default().bold()),
            Span::raw(" Close"),
        ]),
        SettingsMode::Categories => Line::from(vec![
            Span::styled("[n]", Style::default().bold()),
            Span::raw(" New  "),
            Span::styled("[d]", Style::default().bold()),
            Span::raw(" Delete  "),
            Span::styled("[j/k]", Style::default().bold()),
            Span::raw(" Navigate  "),
            Span::styled("[1/2]", Style::default().bold()),
            Span::raw(" Mode  "),
            Span::styled("[Esc]", Style::default().bold()),
            Span::raw(" Close"),
        ]),
    };
    frame.render_widget(Paragraph::new(controls).centered().dark_gray(), chunks[2]);
}

/// Render the mode tab selector
fn render_mode_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let timer_style = if app.settings.mode == SettingsMode::Timer {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().dark_gray()
    };
    let cat_style = if app.settings.mode == SettingsMode::Categories {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().dark_gray()
    };

    let tabs = Line::from(vec![
        Span::styled("[1] Timer", timer_style),
        Span::raw("   "),
        Span::styled("[2] Categories", cat_style),
    ]);
    frame.render_widget(Paragraph::new(tabs).centered(), area);
}

/// Render timer settings content
fn render_timer_settings(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(2), // Work duration
        Constraint::Length(2), // Short break
        Constraint::Length(2), // Long break
        Constraint::Length(2), // Sessions until long break
    ])
    .split(area);

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
}

/// Render category settings content
fn render_category_settings(frame: &mut Frame, area: Rect, app: &App) {
    match app.settings.category_field {
        CategoryField::List => render_category_list(frame, area, app),
        CategoryField::Name | CategoryField::Color => render_category_form(frame, area, app),
    }
}

/// Render the category list
fn render_category_list(frame: &mut Frame, area: Rect, app: &App) {
    let lines: Vec<Line> = app
        .data
        .categories
        .iter()
        .enumerate()
        .map(|(i, cat)| {
            let is_selected = i == app.settings.category_list_index;
            let prefix = if is_selected { "> " } else { "  " };
            let style = if is_selected {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default()
            };

            Line::from(vec![
                Span::styled(prefix, style),
                Span::styled("■ ", Style::default().fg(cat.color)),
                Span::styled(&cat.name, style),
            ])
        })
        .collect();

    if lines.is_empty() {
        frame.render_widget(
            Paragraph::new("No categories. Press [n] to create one.")
                .centered()
                .dark_gray(),
            area,
        );
    } else {
        frame.render_widget(Paragraph::new(lines), area);
    }
}

/// Render the new category form (placeholder for commit 8)
fn render_category_form(frame: &mut Frame, area: Rect, _app: &App) {
    frame.render_widget(
        Paragraph::new("New category form...").centered().dark_gray(),
        area,
    );
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
