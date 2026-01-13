mod detail;
mod history;
mod input;
mod settings;
mod stats;
mod timer;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::Paragraph,
};

use crate::app::{App, NotificationLevel};

pub use detail::render_detail_modal;
pub use history::render_history;
pub use input::render_input_modal;
pub use settings::render_settings_modal;
pub use stats::render_stats;
pub use timer::render_timer;

/// Render the footer area with either a notification or navigation text
pub fn render_footer(frame: &mut Frame, area: Rect, app: &App, nav_text: &str) {
    if let Some(ref n) = app.notification {
        let color = match n.level {
            NotificationLevel::Warning => Color::Yellow,
            NotificationLevel::Error => Color::Red,
        };
        frame.render_widget(
            Paragraph::new(n.message.as_str())
                .centered()
                .style(Style::default().fg(color).bold()),
            area,
        );
    } else {
        frame.render_widget(Paragraph::new(nav_text).centered().dark_gray(), area);
    }
}
