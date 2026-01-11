use chrono::{Datelike, Local, TimeZone};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, widgets::ListState};

use crate::config::TICK_RATE;
use crate::db::{self, Database};
use crate::event::{AppEvent, poll_event};
use crate::models::{Category, CategoryStat, Config, DurationSecs, Session, Timestamp};
use crate::timer::PomodoroTimer;
use crate::ui::{
    render_history, render_input_modal, render_settings_modal, render_stats, render_timer,
};

/// The current view/screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Timer,
    History,
    Stats,
}

/// The time period for statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatsPeriod {
    #[default]
    Day,
    Week,
    Month,
    Year,
}

impl StatsPeriod {
    /// Get the start and end timestamps for this period
    pub fn time_range(&self) -> (i64, i64) {
        let now = Local::now();
        let today_start = Local
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
            .unwrap();

        let (start, end) = match self {
            StatsPeriod::Day => (today_start, now),
            StatsPeriod::Week => {
                let days_since_monday = now.weekday().num_days_from_monday();
                let week_start = today_start - chrono::Duration::days(days_since_monday as i64);
                (week_start, now)
            }
            StatsPeriod::Month => {
                let month_start = Local
                    .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
                    .unwrap();
                (month_start, now)
            }
            StatsPeriod::Year => {
                let year_start = Local.with_ymd_and_hms(now.year(), 1, 1, 0, 0, 0).unwrap();
                (year_start, now)
            }
        };

        (start.timestamp(), end.timestamp())
    }

    pub fn next(&self) -> Self {
        match self {
            StatsPeriod::Day => StatsPeriod::Week,
            StatsPeriod::Week => StatsPeriod::Month,
            StatsPeriod::Month => StatsPeriod::Year,
            StatsPeriod::Year => StatsPeriod::Day,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            StatsPeriod::Day => StatsPeriod::Year,
            StatsPeriod::Week => StatsPeriod::Day,
            StatsPeriod::Month => StatsPeriod::Week,
            StatsPeriod::Year => StatsPeriod::Month,
        }
    }
}

/// Which input field is focused in the input modal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputField {
    #[default]
    Name,
    Description,
    Category,
}

impl InputField {
    pub fn next(&self) -> Self {
        match self {
            InputField::Name => InputField::Description,
            InputField::Description => InputField::Category,
            InputField::Category => InputField::Name,
        }
    }
}

/// Which field is focused in the settings modal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsField {
    #[default]
    WorkDuration,
    ShortBreak,
    LongBreak,
    SessionsUntilLong,
}

/// The current modal state - only one modal can be open at a time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModalState {
    #[default]
    None,
    Input,
    Settings,
}

/// The current session lifecycle state
#[derive(Debug, Clone, Default)]
pub enum SessionPhase {
    /// No session created
    #[default]
    Inactive,
    /// Session created but not currently in a work period
    Ready(Session),
    /// Session in active work period
    Active {
        session: Session,
        start_time: Timestamp,
    },
}

impl SettingsField {
    pub fn next(&self) -> Self {
        match self {
            SettingsField::WorkDuration => SettingsField::ShortBreak,
            SettingsField::ShortBreak => SettingsField::LongBreak,
            SettingsField::LongBreak => SettingsField::SessionsUntilLong,
            SettingsField::SessionsUntilLong => SettingsField::WorkDuration,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            SettingsField::WorkDuration => SettingsField::SessionsUntilLong,
            SettingsField::ShortBreak => SettingsField::WorkDuration,
            SettingsField::LongBreak => SettingsField::ShortBreak,
            SettingsField::SessionsUntilLong => SettingsField::LongBreak,
        }
    }
}

/// The main application state
pub struct App {
    pub running: bool,
    pub view: View,
    pub modal: ModalState,
    pub timer: PomodoroTimer,
    pub session_phase: SessionPhase,

    // Input modal state
    pub input_field: InputField,
    pub input_name: String,
    pub input_description: String,
    pub selected_category: usize,
    pub categories: Vec<Category>,

    // Settings modal state
    pub settings_field: SettingsField,
    pub settings_editing_value: String,
    pub config: Config,
    pub editing_config: Config,

    // History view state
    pub sessions: Vec<Session>,
    pub history_state: ListState,

    // Stats view state
    pub stats_period: StatsPeriod,
    pub category_stats: Vec<CategoryStat>,

    // Database
    db: Option<Database>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: false,
            view: View::Timer,
            modal: ModalState::None,
            timer: PomodoroTimer::new(),
            session_phase: SessionPhase::Inactive,
            input_field: InputField::Name,
            input_name: String::new(),
            input_description: String::new(),
            selected_category: 0,
            categories: Category::defaults(),
            settings_field: SettingsField::WorkDuration,
            settings_editing_value: String::new(),
            config: Config::default(),
            editing_config: Config::default(),
            sessions: Vec::new(),
            history_state: ListState::default(),
            stats_period: StatsPeriod::Day,
            category_stats: Vec::new(),
            db: None,
        }
    }
}

impl App {
    /// Create a new application instance
    pub fn new() -> color_eyre::Result<Self> {
        let mut app = Self::default();

        // Open database and load data
        match Database::open() {
            Ok(database) => {
                // Load categories
                if let Ok(cats) = db::get_categories(&database.conn)
                    && !cats.is_empty()
                {
                    app.categories = cats;
                }

                // Load config and apply to timer
                if let Ok(config) = db::get_config(&database.conn) {
                    app.timer.apply_config(&config);
                    app.config = config;
                }

                app.db = Some(database);
                app.refresh_data();
            }
            Err(e) => {
                eprintln!("Warning: Could not open database: {}", e);
                // Continue without database
            }
        }

        Ok(app)
    }

    /// Get the current session, if any
    pub fn current_session(&self) -> Option<&Session> {
        match &self.session_phase {
            SessionPhase::Ready(s) | SessionPhase::Active { session: s, .. } => Some(s),
            SessionPhase::Inactive => None,
        }
    }

    /// Check if a session exists (ready or active)
    fn has_session(&self) -> bool {
        !matches!(self.session_phase, SessionPhase::Inactive)
    }

    /// Run the application's main loop
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(event) = poll_event(TICK_RATE)? {
                match event {
                    AppEvent::Key(key) => self.handle_key_event(key),
                    AppEvent::Tick => self.handle_tick(),
                }
            }
        }

        Ok(())
    }

    /// Render the current view
    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        match self.view {
            View::Timer => render_timer(frame, area, self),
            View::History => render_history(frame, area, self),
            View::Stats => render_stats(frame, area, self),
        }

        // Render modal on top if visible
        match self.modal {
            ModalState::None => {}
            ModalState::Input => render_input_modal(frame, area, self),
            ModalState::Settings => render_settings_modal(frame, area, self),
        }
    }

    /// Handle a key event
    fn handle_key_event(&mut self, key: KeyEvent) {
        // Handle modal input first
        match self.modal {
            ModalState::Settings => {
                self.handle_settings_modal_key(key);
                return;
            }
            ModalState::Input => {
                self.handle_input_modal_key(key);
                return;
            }
            ModalState::None => {}
        }

        // Global keys
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('q')) | (_, KeyCode::Esc) => self.quit(),
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => self.quit(),
            (_, KeyCode::Tab) => self.view = View::Timer,
            (_, KeyCode::Char('h')) if self.view != View::History => {
                self.view = View::History;
                self.refresh_data();
            }
            (_, KeyCode::Char('t')) if self.view != View::Stats => {
                self.view = View::Stats;
                self.refresh_data();
            }
            _ => {
                // View-specific keys
                match self.view {
                    View::Timer => self.handle_timer_key(key),
                    View::History => self.handle_history_key(key),
                    View::Stats => self.handle_stats_key(key),
                }
            }
        }
    }

    /// Handle timer view keys
    fn handle_timer_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('s') => {
                if self.timer.phase.is_break() {
                    self.timer.skip_break();
                } else if self.timer.is_paused() {
                    self.timer.start();
                } else if self.timer.is_idle() && self.has_session() {
                    self.start_timer();
                }
            }
            KeyCode::Char('p') => {
                if self.timer.is_running() {
                    self.timer.pause();
                }
            }
            KeyCode::Char('r') => {
                self.timer.reset();
            }
            KeyCode::Char('n') => {
                self.modal = ModalState::Input;
                self.input_field = InputField::Name;
                self.input_name.clear();
                self.input_description.clear();
                self.selected_category = 0;
            }
            KeyCode::Char('c') => {
                self.modal = ModalState::Settings;
                self.settings_field = SettingsField::WorkDuration;
                self.editing_config = self.config.clone();
                self.settings_editing_value = self.get_editing_field_value();
            }
            _ => {}
        }
    }

    /// Handle history view keys
    fn handle_history_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let len = self.sessions.len();
                if len > 0 {
                    let i = self.history_state.selected().map(|i| (i + 1) % len);
                    self.history_state.select(i.or(Some(0)));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let len = self.sessions.len();
                if len > 0 {
                    let i = self
                        .history_state
                        .selected()
                        .map(|i| if i == 0 { len - 1 } else { i - 1 });
                    self.history_state.select(i.or(Some(0)));
                }
            }
            KeyCode::Char('d') => {
                // Delete selected session
                if let Some(idx) = self.history_state.selected()
                    && idx < self.sessions.len()
                    && let Some(id) = self.sessions[idx].id
                    && let Some(ref db) = self.db
                {
                    if let Err(e) = db::queries::delete_session(&db.conn, id) {
                        eprintln!("Failed to delete session: {}", e);
                    }
                    self.refresh_data();
                }
            }
            _ => {}
        }
    }

    /// Handle stats view keys
    fn handle_stats_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                self.stats_period = self.stats_period.prev();
                self.refresh_data();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.stats_period = self.stats_period.next();
                self.refresh_data();
            }
            _ => {}
        }
    }

    /// Handle input modal keys
    fn handle_input_modal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.modal = ModalState::None;
            }
            KeyCode::Tab => {
                self.input_field = self.input_field.next();
            }
            KeyCode::Enter => {
                if !self.input_name.is_empty() {
                    self.create_session();
                    self.modal = ModalState::None;
                    self.start_timer();
                }
            }
            KeyCode::Backspace => match self.input_field {
                InputField::Name => {
                    self.input_name.pop();
                }
                InputField::Description => {
                    self.input_description.pop();
                }
                InputField::Category => {}
            },
            KeyCode::Left => {
                if self.input_field == InputField::Category {
                    if self.selected_category == 0 {
                        self.selected_category = self.categories.len() - 1;
                    } else {
                        self.selected_category -= 1;
                    }
                }
            }
            KeyCode::Right => {
                if self.input_field == InputField::Category {
                    self.selected_category = (self.selected_category + 1) % self.categories.len();
                }
            }
            KeyCode::Char(c) => match self.input_field {
                InputField::Name => {
                    self.input_name.push(c);
                }
                InputField::Description => {
                    self.input_description.push(c);
                }
                InputField::Category => {}
            },
            _ => {}
        }
    }

    /// Handle settings modal keys
    fn handle_settings_modal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.modal = ModalState::None;
            }
            KeyCode::Tab | KeyCode::Down => {
                self.apply_editing_value();
                self.settings_field = self.settings_field.next();
                self.settings_editing_value = self.get_editing_field_value();
            }
            KeyCode::Up => {
                self.apply_editing_value();
                self.settings_field = self.settings_field.prev();
                self.settings_editing_value = self.get_editing_field_value();
            }
            KeyCode::Enter => {
                self.save_settings();
                self.modal = ModalState::None;
            }
            KeyCode::Backspace => {
                self.settings_editing_value.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.settings_editing_value.push(c);
            }
            _ => {}
        }
    }

    /// Get the current value for the selected settings field from editing_config
    fn get_editing_field_value(&self) -> String {
        match self.settings_field {
            SettingsField::WorkDuration => {
                (self.editing_config.work_duration_secs / 60).to_string()
            }
            SettingsField::ShortBreak => (self.editing_config.short_break_secs / 60).to_string(),
            SettingsField::LongBreak => (self.editing_config.long_break_secs / 60).to_string(),
            SettingsField::SessionsUntilLong => {
                self.editing_config.sessions_until_long_break.to_string()
            }
        }
    }

    /// Apply the current editing value to editing_config
    fn apply_editing_value(&mut self) {
        if let Ok(value) = self.settings_editing_value.parse::<i64>()
            && value > 0
        {
            match self.settings_field {
                SettingsField::WorkDuration => {
                    self.editing_config.work_duration_secs = value * 60;
                }
                SettingsField::ShortBreak => {
                    self.editing_config.short_break_secs = value * 60;
                }
                SettingsField::LongBreak => {
                    self.editing_config.long_break_secs = value * 60;
                }
                SettingsField::SessionsUntilLong => {
                    self.editing_config.sessions_until_long_break = value;
                }
            }
        }
    }

    /// Save all settings to the database and apply to timer
    fn save_settings(&mut self) {
        // Apply the current field's value to editing_config
        self.apply_editing_value();

        // Commit editing_config to config
        self.config = self.editing_config.clone();

        // Apply to timer
        self.timer.apply_config(&self.config);

        // Save to database
        if let Some(ref db) = self.db
            && let Err(e) = db::save_config(&db.conn, &self.config)
        {
            eprintln!("Failed to save config: {}", e);
        }
    }

    /// Handle a timer tick
    fn handle_tick(&mut self) {
        if self.timer.is_running() && self.timer.is_finished() {
            // Phase completed - ring terminal bell
            print!("\x07");
            let _ = std::io::Write::flush(&mut std::io::stdout());

            if self.timer.phase == crate::timer::TimerPhase::Work {
                // Save the completed work session
                self.complete_session();
            }
            self.timer.advance_phase();

            // Auto-start the next phase
            self.timer.start();
        }
    }

    /// Start the timer
    fn start_timer(&mut self) {
        let phase = std::mem::take(&mut self.session_phase);

        self.session_phase = match phase {
            SessionPhase::Ready(session) => SessionPhase::Active {
                session,
                start_time: Timestamp::now(),
            },
            SessionPhase::Active { session, .. } => SessionPhase::Active {
                session,
                start_time: Timestamp::now(),
            },
            SessionPhase::Inactive => SessionPhase::Inactive,
        };

        self.timer.start();
    }

    /// Create a new session from input
    fn create_session(&mut self) {
        let category = self.categories[self.selected_category].name.clone();
        let description = if self.input_description.is_empty() {
            None
        } else {
            Some(self.input_description.clone())
        };

        let session = Session {
            id: None,
            name: self.input_name.clone(),
            description,
            category,
            started_at: Timestamp(0),
            ended_at: Timestamp(0),
            duration_secs: DurationSecs(0),
        };

        self.session_phase = SessionPhase::Ready(session);
    }

    /// Complete the current session and save to database
    fn complete_session(&mut self) {
        let phase = std::mem::take(&mut self.session_phase);

        self.session_phase = match phase {
            SessionPhase::Active {
                mut session,
                start_time,
            } => {
                let end_time = Timestamp::now();
                session.started_at = start_time;
                session.ended_at = end_time;
                session.duration_secs = end_time - start_time;

                if let Some(ref db) = self.db
                    && let Err(e) = db::save_session(&db.conn, &session)
                {
                    eprintln!("Failed to save session: {}", e);
                }

                SessionPhase::Ready(session)
            }
            other => other,
        };
    }

    /// Refresh data from database
    fn refresh_data(&mut self) {
        if let Some(ref db) = self.db {
            // Load sessions for history (last 30 days)
            let now = Local::now().timestamp();
            let thirty_days_ago = now - (30 * 24 * 60 * 60);
            if let Ok(sessions) = db::get_sessions_in_range(&db.conn, thirty_days_ago, now) {
                self.sessions = sessions;
            }

            // Load category stats for current period
            let (start, end) = self.stats_period.time_range();
            if let Ok(stats) = db::get_time_by_category(&db.conn, start, end) {
                self.category_stats = stats;
            }
        }
    }

    /// Quit the application
    fn quit(&mut self) {
        self.running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_editing_buffer_isolation() {
        let mut app = App::default();
        app.config.work_duration_secs = 25 * 60;

        // Simulate opening settings modal
        app.editing_config = app.config.clone();
        app.settings_field = SettingsField::WorkDuration;
        app.settings_editing_value = "30".to_string();

        // Apply the editing value (simulates navigation)
        app.apply_editing_value();

        // editing_config should be updated, config should not
        assert_eq!(app.editing_config.work_duration_secs, 30 * 60);
        assert_eq!(app.config.work_duration_secs, 25 * 60);
    }

    #[test]
    fn test_settings_save_commits_all_changes() {
        let mut app = App::default();
        app.config.work_duration_secs = 25 * 60;
        app.config.short_break_secs = 5 * 60;

        // Simulate editing multiple fields
        app.editing_config = app.config.clone();

        // Edit work duration
        app.settings_field = SettingsField::WorkDuration;
        app.settings_editing_value = "30".to_string();
        app.apply_editing_value();

        // Navigate to short break and edit it
        app.settings_field = SettingsField::ShortBreak;
        app.settings_editing_value = "10".to_string();
        app.apply_editing_value();

        // Save should commit both changes
        app.settings_field = SettingsField::ShortBreak;
        app.settings_editing_value = "10".to_string();
        app.save_settings();

        assert_eq!(app.config.work_duration_secs, 30 * 60);
        assert_eq!(app.config.short_break_secs, 10 * 60);
    }

    #[test]
    fn test_settings_cancel_discards_changes() {
        let mut app = App::default();
        app.config.work_duration_secs = 25 * 60;

        // Simulate editing
        app.editing_config = app.config.clone();
        app.settings_field = SettingsField::WorkDuration;
        app.settings_editing_value = "30".to_string();
        app.apply_editing_value();

        // Cancel (just close modal without saving)
        app.modal = ModalState::None;

        // Config should be unchanged
        assert_eq!(app.config.work_duration_secs, 25 * 60);
    }
}
