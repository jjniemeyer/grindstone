use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph},
};

use crate::app::{App, StatsPeriod};
use crate::models::CategoryStat;

/// Render the statistics view
pub fn render_stats(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Title
        Constraint::Length(3), // Period selector
        Constraint::Min(1),    // Chart area
        Constraint::Length(3), // Summary
        Constraint::Length(2), // Controls
    ])
    .split(area);

    // Title
    let title = Line::from("Time Statistics").bold().blue().centered();
    frame.render_widget(
        Paragraph::new(title).block(Block::default().borders(Borders::BOTTOM)),
        chunks[0],
    );

    // Period selector
    let periods = ["Day", "Week", "Month", "Year"];
    let selected_idx = match app.stats_period {
        StatsPeriod::Day => 0,
        StatsPeriod::Week => 1,
        StatsPeriod::Month => 2,
        StatsPeriod::Year => 3,
    };

    let period_spans: Vec<Span> = periods
        .iter()
        .enumerate()
        .flat_map(|(i, p)| {
            let style = if i == selected_idx {
                Style::default().bold().fg(Color::Cyan)
            } else {
                Style::default().dark_gray()
            };
            vec![
                Span::raw(if i == selected_idx { "[ " } else { "  " }),
                Span::styled(*p, style),
                Span::raw(if i == selected_idx { " ]" } else { "  " }),
            ]
        })
        .collect();

    frame.render_widget(
        Paragraph::new(Line::from(period_spans)).centered(),
        chunks[1],
    );

    // Chart area - horizontal bar chart showing time by category
    let chart_chunks = Layout::horizontal([
        Constraint::Percentage(60), // Bar chart
        Constraint::Percentage(40), // Legend
    ])
    .split(chunks[2]);

    render_bar_chart(frame, chart_chunks[0], &app.category_stats);
    render_legend(frame, chart_chunks[1], &app.category_stats);

    // Summary stats
    let total_secs: i64 = app.category_stats.iter().map(|s| s.total_seconds).sum();
    let total_hours = total_secs / 3600;
    let total_mins = (total_secs % 3600) / 60;
    let session_count = app.sessions.len();

    let summary = format!(
        "Total: {}h {}m  |  Sessions: {}",
        total_hours, total_mins, session_count
    );
    frame.render_widget(
        Paragraph::new(summary)
            .centered()
            .block(Block::default().borders(Borders::TOP)),
        chunks[3],
    );

    // Controls
    let controls = "[</> or h/l] Change Period  [Tab] Timer  [h] History  [q] Quit";
    frame.render_widget(
        Paragraph::new(controls)
            .centered()
            .dark_gray()
            .block(Block::default().borders(Borders::TOP)),
        chunks[4],
    );
}

fn render_bar_chart(frame: &mut Frame, area: Rect, stats: &[CategoryStat]) {
    if stats.is_empty() {
        frame.render_widget(
            Paragraph::new("No data for this period")
                .centered()
                .dark_gray(),
            area,
        );
        return;
    }

    let max_secs = stats.iter().map(|s| s.total_seconds).max().unwrap_or(1);

    let colors = [
        Color::Red,
        Color::Cyan,
        Color::Blue,
        Color::Green,
        Color::Yellow,
        Color::Magenta,
    ];

    let bars: Vec<Bar> = stats
        .iter()
        .enumerate()
        .map(|(i, stat)| {
            let hours = stat.total_seconds / 3600;
            let mins = (stat.total_seconds % 3600) / 60;
            let label = if hours > 0 {
                format!("{}h{}m", hours, mins)
            } else {
                format!("{}m", mins)
            };
            Bar::default()
                .value(stat.total_seconds as u64)
                .label(Line::from(stat.name.clone()))
                .text_value(label)
                .style(Style::default().fg(colors[i % colors.len()]))
        })
        .collect();

    let chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Time by Category"),
        )
        .bar_width(3)
        .bar_gap(1)
        .data(BarGroup::default().bars(&bars))
        .max(max_secs as u64);

    frame.render_widget(chart, area);
}

fn render_legend(frame: &mut Frame, area: Rect, stats: &[CategoryStat]) {
    let total_secs: i64 = stats.iter().map(|s| s.total_seconds).sum();
    if total_secs == 0 {
        return;
    }

    let colors = [
        Color::Red,
        Color::Cyan,
        Color::Blue,
        Color::Green,
        Color::Yellow,
        Color::Magenta,
    ];

    let lines: Vec<Line> = stats
        .iter()
        .enumerate()
        .map(|(i, stat)| {
            let hours = stat.total_seconds / 3600;
            let mins = (stat.total_seconds % 3600) / 60;
            let pct = (stat.total_seconds as f64 / total_secs as f64) * 100.0;
            let time_str = if hours > 0 {
                format!("{}h {}m", hours, mins)
            } else {
                format!("{}m", mins)
            };
            Line::from(vec![
                Span::styled("■ ", Style::default().fg(colors[i % colors.len()])),
                Span::styled(
                    format!("{:<12}", stat.name),
                    Style::default().fg(colors[i % colors.len()]),
                ),
                Span::raw(format!("{:>8}  ({:.0}%)", time_str, pct)),
            ])
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Breakdown")),
        area,
    );
}
