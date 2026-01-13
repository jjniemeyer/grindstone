use chrono::{Datelike, Local, TimeZone};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::{error, warn};
use ratatui::{DefaultTerminal, Frame, widgets::ListState};

use crate::clock::{Clock, SystemClock};
use crate::config::TICK_RATE;
use crate::db::{Database, DatabaseOps};
use crate::event::{AppEvent, poll_event};
use crate::models::{
    BoundedString, Category, CategoryId, CategoryStat, Config, DurationSecs, Session, Timestamp,
};
use crate::timer::PomodoroTimer;
use crate::ui::{
    render_detail_modal, render_history, render_input_modal, render_settings_modal, render_stats,
    render_timer,
};
use crate::validation::{
    validate_new_category_name, validate_session_name, validate_update_category_name,
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
    /// Get the start and end timestamps for this period using a clock
    pub fn time_range_with_clock(&self, clock: &dyn Clock) -> (i64, i64) {
        let now = clock.now_datetime();
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

/// The chart type for statistics visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartType {
    #[default]
    Bar,
    Pie,
}

impl ChartType {
    pub fn toggle(self) -> Self {
        match self {
            ChartType::Bar => ChartType::Pie,
            ChartType::Pie => ChartType::Bar,
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

/// Which field is focused in the settings modal (timer mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsField {
    #[default]
    WorkDuration,
    ShortBreak,
    LongBreak,
    SessionsUntilLong,
}

/// Which mode/tab is active in the settings modal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsMode {
    #[default]
    Timer,
    Categories,
}

/// Which field is focused in category editing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CategoryField {
    #[default]
    List,
    Name,
    Color,
}

/// The current modal state - only one modal can be open at a time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModalState {
    #[default]
    None,
    Input,
    Settings,
    Detail,
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

/// Notification severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Warning,
    Error,
}

/// A notification message to display to the user
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
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

/// State for the input modal
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub field: InputField,
    pub name: BoundedString<100>,
    pub description: BoundedString<500>,
    pub selected_category: usize,
}

/// State for the settings modal
#[derive(Debug, Clone, Default)]
pub struct SettingsState {
    pub mode: SettingsMode,
    // Timer mode fields
    pub field: SettingsField,
    pub editing_value: String,
    pub editing_config: Config,
    // Category mode fields
    pub category_field: CategoryField,
    pub category_list_index: usize,
    pub new_category_name: BoundedString<50>,
    pub new_category_color: BoundedString<7>,
    pub editing_category_id: Option<CategoryId>, // Some when editing, None when creating
}

/// State for the session detail modal
#[derive(Debug, Clone, Default)]
pub struct DetailState {
    pub selected_session_index: usize,
}

/// Persisted application data
#[derive(Debug, Clone, Default)]
pub struct AppData {
    pub categories: Vec<Category>,
    pub config: Config,
    pub sessions: Vec<Session>,
    pub history_state: ListState,
    pub stats_period: StatsPeriod,
    pub chart_type: ChartType,
    pub category_stats: Vec<CategoryStat>,
}

/// The main application state
pub struct App {
    pub running: bool,
    pub view: View,
    pub modal: ModalState,
    pub timer: PomodoroTimer,
    pub session_phase: SessionPhase,
    pub input: InputState,
    pub settings: SettingsState,
    pub detail: DetailState,
    pub data: AppData,
    pub notification: Option<Notification>,
    db: Option<Box<dyn DatabaseOps>>,
    clock: Box<dyn Clock>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: false,
            view: View::Timer,
            modal: ModalState::None,
            timer: PomodoroTimer::new(),
            session_phase: SessionPhase::Inactive,
            input: InputState::default(),
            settings: SettingsState::default(),
            detail: DetailState::default(),
            data: AppData {
                categories: Category::defaults(),
                config: Config::default(),
                sessions: Vec::new(),
                history_state: ListState::default(),
                stats_period: StatsPeriod::Day,
                chart_type: ChartType::Bar,
                category_stats: Vec::new(),
            },
            notification: None,
            db: None,
            clock: Box::new(SystemClock),
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
                let db: Box<dyn DatabaseOps> = Box::new(database);

                // Load categories
                if let Ok(cats) = db.get_categories()
                    && !cats.is_empty()
                {
                    app.data.categories = cats;
                }

                // Load config and apply to timer
                if let Ok(config) = db.get_config() {
                    app.timer.apply_config(&config);
                    app.data.config = config;
                }

                app.db = Some(db);
                app.refresh_data();
            }
            Err(e) => {
                warn!("Could not open database: {}", e);
                app.notify(NotificationLevel::Warning, "Running without database");
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
            ModalState::Detail => render_detail_modal(frame, area, self),
        }
    }

    /// Handle a key event
    fn handle_key_event(&mut self, key: KeyEvent) {
        // Clear any notification on key press
        self.notification = None;

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
            ModalState::Detail => {} // TODO: handle detail modal keys
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
            KeyCode::Char('x') => {
                if self.timer.is_running() || self.timer.is_paused() {
                    self.stop_session();
                }
            }
            KeyCode::Char('n') => {
                self.modal = ModalState::Input;
                self.input.field = InputField::Name;
                self.input.name.clear();
                self.input.description.clear();
                self.input.selected_category = 0;
            }
            KeyCode::Char('c') => {
                self.modal = ModalState::Settings;
                self.settings.field = SettingsField::WorkDuration;
                self.settings.editing_config = self.data.config.clone();
                self.settings.editing_value = self.get_editing_field_value();
            }
            _ => {}
        }
    }

    /// Count the number of rendered list items in history view (sessions + date headers)
    fn count_history_list_items(sessions: &[Session]) -> usize {
        let mut count = 0;
        let mut current_date: Option<(i32, u32, u32)> = None;

        for session in sessions {
            let dt = session.start_datetime();
            let date = (dt.year(), dt.month(), dt.day());

            // Add header for new date
            if current_date != Some(date) {
                current_date = Some(date);
                count += 1;
            }
            // Add session item
            count += 1;
        }
        count.max(1) // At least 1 for "No sessions" message
    }

    /// Handle history view keys
    fn handle_history_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let len = Self::count_history_list_items(&self.data.sessions);
                if len > 0 {
                    let i = self.data.history_state.selected().map(|i| (i + 1) % len);
                    self.data.history_state.select(i.or(Some(0)));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let len = Self::count_history_list_items(&self.data.sessions);
                if len > 0 {
                    let i = self
                        .data
                        .history_state
                        .selected()
                        .map(|i| if i == 0 { len - 1 } else { i - 1 });
                    self.data.history_state.select(i.or(Some(0)));
                }
            }
            KeyCode::Char('d') => {
                // Delete selected session
                if let Some(idx) = self.data.history_state.selected()
                    && idx < self.data.sessions.len()
                    && let Some(id) = self.data.sessions[idx].id
                    && let Some(ref db) = self.db
                {
                    if let Err(e) = db.delete_session(id) {
                        warn!("Failed to delete session: {}", e);
                        self.notify(NotificationLevel::Warning, "Failed to delete session");
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
                self.data.stats_period = self.data.stats_period.prev();
                self.refresh_data();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.data.stats_period = self.data.stats_period.next();
                self.refresh_data();
            }
            KeyCode::Char('v') => {
                self.data.chart_type = self.data.chart_type.toggle();
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
                self.input.field = self.input.field.next();
            }
            KeyCode::Enter => {
                if validate_session_name(self.input.name.as_ref()) {
                    self.create_session();
                    self.modal = ModalState::None;
                    self.start_timer();
                }
            }
            KeyCode::Backspace => match self.input.field {
                InputField::Name => {
                    self.input.name.pop();
                }
                InputField::Description => {
                    self.input.description.pop();
                }
                InputField::Category => {}
            },
            KeyCode::Left => {
                if self.input.field == InputField::Category {
                    if self.input.selected_category == 0 {
                        self.input.selected_category = self.data.categories.len() - 1;
                    } else {
                        self.input.selected_category -= 1;
                    }
                }
            }
            KeyCode::Right => {
                if self.input.field == InputField::Category {
                    self.input.selected_category =
                        (self.input.selected_category + 1) % self.data.categories.len();
                }
            }
            KeyCode::Char(c) => match self.input.field {
                InputField::Name => {
                    self.input.name.push(c);
                }
                InputField::Description => {
                    self.input.description.push(c);
                }
                InputField::Category => {}
            },
            _ => {}
        }
    }

    /// Handle settings modal keys
    fn handle_settings_modal_key(&mut self, key: KeyEvent) {
        // Mode switching (works in both modes when not editing)
        if self.settings.category_field == CategoryField::List {
            match key.code {
                KeyCode::Char('1') => {
                    self.settings.mode = SettingsMode::Timer;
                    return;
                }
                KeyCode::Char('2') => {
                    self.settings.mode = SettingsMode::Categories;
                    return;
                }
                _ => {}
            }
        }

        match self.settings.mode {
            SettingsMode::Timer => self.handle_timer_settings_key(key),
            SettingsMode::Categories => self.handle_category_settings_key(key),
        }
    }

    /// Handle timer settings mode keys
    fn handle_timer_settings_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.modal = ModalState::None;
            }
            KeyCode::Tab | KeyCode::Down => {
                self.apply_editing_value();
                self.settings.field = self.settings.field.next();
                self.settings.editing_value = self.get_editing_field_value();
            }
            KeyCode::Up => {
                self.apply_editing_value();
                self.settings.field = self.settings.field.prev();
                self.settings.editing_value = self.get_editing_field_value();
            }
            KeyCode::Enter => {
                self.save_settings();
                self.modal = ModalState::None;
            }
            KeyCode::Backspace => {
                self.settings.editing_value.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.settings.editing_value.push(c);
            }
            _ => {}
        }
    }

    /// Handle category settings mode keys
    fn handle_category_settings_key(&mut self, key: KeyEvent) {
        match self.settings.category_field {
            CategoryField::List => self.handle_category_list_key(key),
            CategoryField::Name | CategoryField::Color => self.handle_category_form_key(key),
        }
    }

    /// Handle keys when browsing the category list
    fn handle_category_list_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.modal = ModalState::None;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let len = self.data.categories.len();
                if len > 0 {
                    self.settings.category_list_index =
                        (self.settings.category_list_index + 1) % len;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let len = self.data.categories.len();
                if len > 0 {
                    if self.settings.category_list_index == 0 {
                        self.settings.category_list_index = len - 1;
                    } else {
                        self.settings.category_list_index -= 1;
                    }
                }
            }
            KeyCode::Char('n') => {
                // Start creating a new category
                self.settings.category_field = CategoryField::Name;
                self.settings.new_category_name.clear();
                self.settings.new_category_color.clear();
                self.settings.editing_category_id = None;
                // Pre-fill with a default color
                for c in "#808080".chars() {
                    self.settings.new_category_color.push(c);
                }
            }
            KeyCode::Char('e') => {
                // Edit selected category
                self.start_editing_category();
            }
            KeyCode::Char('d') => {
                self.delete_selected_category();
            }
            _ => {}
        }
    }

    /// Handle keys when editing the new category form
    fn handle_category_form_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Cancel and return to list
                self.settings.category_field = CategoryField::List;
            }
            KeyCode::Tab => {
                // Cycle between name and color fields
                self.settings.category_field = match self.settings.category_field {
                    CategoryField::Name => CategoryField::Color,
                    CategoryField::Color => CategoryField::Name,
                    CategoryField::List => CategoryField::List,
                };
            }
            KeyCode::Enter => {
                self.save_category();
            }
            KeyCode::Backspace => match self.settings.category_field {
                CategoryField::Name => {
                    self.settings.new_category_name.pop();
                }
                CategoryField::Color => {
                    self.settings.new_category_color.pop();
                }
                CategoryField::List => {}
            },
            KeyCode::Char(c) => match self.settings.category_field {
                CategoryField::Name => {
                    self.settings.new_category_name.push(c);
                }
                CategoryField::Color => {
                    // Only allow hex color characters
                    if c == '#' || c.is_ascii_hexdigit() {
                        self.settings.new_category_color.push(c);
                    }
                }
                CategoryField::List => {}
            },
            _ => {}
        }
    }

    /// Start editing the selected category
    fn start_editing_category(&mut self) {
        let idx = self.settings.category_list_index;
        if idx >= self.data.categories.len() {
            return;
        }

        let category = &self.data.categories[idx];

        // Pre-fill the form with current values
        self.settings.new_category_name.clear();
        for c in category.name.chars() {
            self.settings.new_category_name.push(c);
        }

        self.settings.new_category_color.clear();
        let color_hex = crate::models::format_hex_color(category.color);
        for c in color_hex.chars() {
            self.settings.new_category_color.push(c);
        }

        self.settings.editing_category_id = category.id;
        self.settings.category_field = CategoryField::Name;
    }

    /// Save category (create new or update existing)
    fn save_category(&mut self) {
        let name = self.settings.new_category_name.to_string();
        let color_str = self.settings.new_category_color.to_string();

        // Parse color (use default if invalid)
        let color = crate::models::parse_hex_color(&color_str);

        if let Some(ref db) = self.db {
            let result = if let Some(id) = self.settings.editing_category_id {
                // Validate for update - find current name to allow keeping it
                let current_name = self
                    .data
                    .categories
                    .iter()
                    .find(|c| c.id == Some(id))
                    .map(|c| c.name.as_str())
                    .unwrap_or("");
                if let Err(msg) =
                    validate_update_category_name(&name, &self.data.categories, current_name)
                {
                    self.notify(NotificationLevel::Warning, msg);
                    return;
                }
                // Update existing category
                db.update_category(id, &name, color)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            } else {
                // Validate for create
                if let Err(msg) = validate_new_category_name(&name, &self.data.categories) {
                    self.notify(NotificationLevel::Warning, msg);
                    return;
                }
                // Create new category
                db.create_category(&name, color)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            };

            match result {
                Ok(()) => {
                    self.refresh_categories();
                    self.settings.category_field = CategoryField::List;
                    self.settings.editing_category_id = None;
                }
                Err(e) => {
                    warn!("Failed to save category: {}", e);
                    self.notify(NotificationLevel::Warning, "Failed to save category");
                }
            }
        } else {
            self.notify(NotificationLevel::Warning, "No database connection");
        }
    }

    /// Delete the currently selected category
    fn delete_selected_category(&mut self) {
        let idx = self.settings.category_list_index;
        if idx >= self.data.categories.len() {
            return;
        }

        let category = &self.data.categories[idx];
        let category_name = category.name.clone();
        let category_id = match category.id {
            Some(id) => id,
            None => {
                self.notify(NotificationLevel::Warning, "Cannot delete default category");
                return;
            }
        };

        if let Some(ref db) = self.db {
            // Check if category is in use
            match db.is_category_in_use(&category_name) {
                Ok(true) => {
                    self.notify(
                        NotificationLevel::Warning,
                        "Cannot delete: category has sessions",
                    );
                    return;
                }
                Ok(false) => {}
                Err(e) => {
                    warn!("Failed to check category usage: {}", e);
                    return;
                }
            }

            // Delete the category
            match db.delete_category(category_id) {
                Ok(_) => {
                    self.refresh_categories();
                    // Adjust selection if needed
                    if self.settings.category_list_index >= self.data.categories.len()
                        && !self.data.categories.is_empty()
                    {
                        self.settings.category_list_index = self.data.categories.len() - 1;
                    }
                }
                Err(e) => {
                    warn!("Failed to delete category: {}", e);
                    self.notify(NotificationLevel::Warning, "Failed to delete category");
                }
            }
        }
    }

    /// Refresh categories from database
    fn refresh_categories(&mut self) {
        if let Some(ref db) = self.db
            && let Ok(cats) = db.get_categories()
            && !cats.is_empty()
        {
            self.data.categories = cats;
        }
    }

    /// Get the current value for the selected settings field from editing_config
    fn get_editing_field_value(&self) -> String {
        match self.settings.field {
            SettingsField::WorkDuration => {
                (self.settings.editing_config.work_duration_secs / 60).to_string()
            }
            SettingsField::ShortBreak => {
                (self.settings.editing_config.short_break_secs / 60).to_string()
            }
            SettingsField::LongBreak => {
                (self.settings.editing_config.long_break_secs / 60).to_string()
            }
            SettingsField::SessionsUntilLong => self
                .settings
                .editing_config
                .sessions_until_long_break
                .to_string(),
        }
    }

    /// Apply the current editing value to editing_config
    fn apply_editing_value(&mut self) {
        if let Ok(value) = self.settings.editing_value.parse::<i64>()
            && value > 0
        {
            match self.settings.field {
                SettingsField::WorkDuration => {
                    self.settings.editing_config.work_duration_secs = value * 60;
                }
                SettingsField::ShortBreak => {
                    self.settings.editing_config.short_break_secs = value * 60;
                }
                SettingsField::LongBreak => {
                    self.settings.editing_config.long_break_secs = value * 60;
                }
                SettingsField::SessionsUntilLong => {
                    self.settings.editing_config.sessions_until_long_break = value;
                }
            }
        }
    }

    /// Save all settings to the database and apply to timer
    fn save_settings(&mut self) {
        // Apply the current field's value to editing_config
        self.apply_editing_value();

        // Validate config before saving
        if !self.settings.editing_config.is_valid() {
            self.notify(
                NotificationLevel::Warning,
                "Invalid settings: all values must be positive",
            );
            return;
        }

        // Commit editing_config to config
        self.data.config = self.settings.editing_config.clone();

        // Apply to timer
        self.timer.apply_config(&self.data.config);

        // Save to database
        if let Some(ref db) = self.db
            && let Err(e) = db.save_config(&self.data.config)
        {
            warn!("Failed to save config: {}", e);
            self.notify(NotificationLevel::Warning, "Failed to save settings");
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
                start_time: Timestamp::from_clock(&*self.clock),
            },
            SessionPhase::Active { session, .. } => SessionPhase::Active {
                session,
                start_time: Timestamp::from_clock(&*self.clock),
            },
            SessionPhase::Inactive => SessionPhase::Inactive,
        };

        self.timer.start();
    }

    /// Create a new session from input
    fn create_session(&mut self) {
        let category = self.data.categories[self.input.selected_category]
            .name
            .clone();
        let description = if self.input.description.is_empty() {
            None
        } else {
            Some(self.input.description.to_string())
        };

        let session = Session::builder()
            .name(self.input.name.to_string())
            .description(description)
            .category(category)
            .started_at(Timestamp::new(0))
            .ended_at(Timestamp::new(0))
            .duration_secs(DurationSecs::new(0))
            .build()
            .expect("session fields validated by UI");

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
                let end_time = Timestamp::from_clock(&*self.clock);
                session.started_at = start_time;
                session.ended_at = end_time;
                // Use configured work duration, not wall-clock time
                session.duration_secs =
                    DurationSecs::new(self.timer.work_duration.as_secs() as i64);

                if let Some(ref db) = self.db
                    && let Err(e) = db.save_session(&session)
                {
                    error!("Failed to save session: {}", e);
                    self.notify(NotificationLevel::Error, "Failed to save session!");
                }

                SessionPhase::Ready(session)
            }
            other => other,
        };
    }

    /// Stop the current session early and save actual elapsed time
    fn stop_session(&mut self) {
        if !matches!(self.session_phase, SessionPhase::Active { .. }) {
            return;
        }

        let elapsed_secs = self.timer.elapsed().as_secs() as i64;
        if elapsed_secs == 0 {
            return;
        }

        let phase = std::mem::take(&mut self.session_phase);

        self.session_phase = match phase {
            SessionPhase::Active {
                mut session,
                start_time,
            } => {
                let end_time = Timestamp::from_clock(&*self.clock);
                session.started_at = start_time;
                session.ended_at = end_time;
                session.duration_secs = DurationSecs::new(elapsed_secs);

                if let Some(ref db) = self.db
                    && let Err(e) = db.save_session(&session)
                {
                    error!("Failed to save session: {}", e);
                    self.notify(NotificationLevel::Error, "Failed to save session!");
                }

                self.timer.reset();
                SessionPhase::Inactive
            }
            other => other,
        };
    }

    /// Refresh data from database
    fn refresh_data(&mut self) {
        if let Some(ref db) = self.db {
            // Load sessions for history (last 30 days)
            let now = self.clock.now_timestamp();
            let thirty_days_ago = now - (30 * 24 * 60 * 60);
            if let Ok(sessions) = db.get_sessions_in_range(thirty_days_ago, now) {
                self.data.sessions = sessions;
            }

            // Load category stats for current period
            let (start, end) = self.data.stats_period.time_range_with_clock(&*self.clock);
            if let Ok(stats) = db.get_time_by_category(start, end) {
                self.data.category_stats = stats;
            }
        }
    }

    /// Set a notification to display to the user
    fn notify(&mut self, level: NotificationLevel, message: impl Into<String>) {
        self.notification = Some(Notification {
            message: message.into(),
            level,
        });
    }

    /// Quit the application
    fn quit(&mut self) {
        self.running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;
    use std::cell::RefCell;

    /// Mock database for testing App without real database
    struct MockDatabase {
        categories: RefCell<Vec<Category>>,
        sessions: RefCell<Vec<Session>>,
        config: RefCell<Config>,
        next_session_id: RefCell<i64>,
        next_category_id: RefCell<i64>,
    }

    impl MockDatabase {
        fn new() -> Self {
            Self {
                categories: RefCell::new(vec![Category {
                    id: None,
                    name: "Default".to_string(),
                    color: Color::Gray,
                }]),
                sessions: RefCell::new(Vec::new()),
                config: RefCell::new(Config::default()),
                next_session_id: RefCell::new(1),
                next_category_id: RefCell::new(1),
            }
        }
    }

    impl DatabaseOps for MockDatabase {
        fn save_session(
            &self,
            session: &Session,
        ) -> crate::error::Result<crate::models::SessionId> {
            let mut sessions = self.sessions.borrow_mut();
            let mut next_id = self.next_session_id.borrow_mut();
            let id = crate::models::SessionId::from(*next_id);
            *next_id += 1;
            let mut session = session.clone();
            session.id = Some(id);
            sessions.push(session);
            Ok(id)
        }

        fn delete_session(&self, id: crate::models::SessionId) -> crate::error::Result<usize> {
            let mut sessions = self.sessions.borrow_mut();
            let len_before = sessions.len();
            sessions.retain(|s| s.id != Some(id));
            Ok(len_before - sessions.len())
        }

        fn get_sessions_in_range(
            &self,
            start: i64,
            end: i64,
        ) -> crate::error::Result<Vec<Session>> {
            let sessions = self.sessions.borrow();
            Ok(sessions
                .iter()
                .filter(|s| {
                    let ts: i64 = s.started_at.into();
                    ts >= start && ts <= end
                })
                .cloned()
                .collect())
        }

        fn get_time_by_category(
            &self,
            _start: i64,
            _end: i64,
        ) -> crate::error::Result<Vec<crate::models::CategoryStat>> {
            Ok(Vec::new())
        }

        fn get_categories(&self) -> crate::error::Result<Vec<Category>> {
            Ok(self.categories.borrow().clone())
        }

        fn create_category(
            &self,
            name: &str,
            color: Color,
        ) -> crate::error::Result<crate::models::CategoryId> {
            let mut categories = self.categories.borrow_mut();
            let mut next_id = self.next_category_id.borrow_mut();
            let id = crate::models::CategoryId::from(*next_id);
            *next_id += 1;
            categories.push(Category {
                id: Some(id),
                name: name.to_string(),
                color,
            });
            Ok(id)
        }

        fn delete_category(&self, id: crate::models::CategoryId) -> crate::error::Result<usize> {
            let mut categories = self.categories.borrow_mut();
            let len_before = categories.len();
            categories.retain(|c| c.id != Some(id));
            Ok(len_before - categories.len())
        }

        fn update_category(
            &self,
            id: crate::models::CategoryId,
            name: &str,
            color: Color,
        ) -> crate::error::Result<usize> {
            let mut categories = self.categories.borrow_mut();
            for cat in categories.iter_mut() {
                if cat.id == Some(id) {
                    cat.name = name.to_string();
                    cat.color = color;
                    return Ok(1);
                }
            }
            Ok(0)
        }

        fn is_category_in_use(&self, name: &str) -> crate::error::Result<bool> {
            let sessions = self.sessions.borrow();
            Ok(sessions.iter().any(|s| s.category == name))
        }

        fn get_config(&self) -> crate::error::Result<Config> {
            Ok(self.config.borrow().clone())
        }

        fn save_config(&self, config: &Config) -> crate::error::Result<()> {
            *self.config.borrow_mut() = config.clone();
            Ok(())
        }
    }

    #[test]
    fn test_app_with_mock_database() {
        let mut app = App::default();
        let mock_db = MockDatabase::new();
        app.db = Some(Box::new(mock_db));

        // App should work with mock database
        app.refresh_data();
        app.refresh_categories();
        assert!(!app.data.categories.is_empty());
    }

    #[test]
    fn test_save_config_with_mock_database() {
        let mut app = App::default();
        let mock_db = MockDatabase::new();
        app.db = Some(Box::new(mock_db));

        // Modify config through settings flow
        app.settings.editing_config = app.data.config.clone();
        app.settings.editing_config.work_duration_secs = 30 * 60;
        app.settings.field = SettingsField::WorkDuration;
        app.settings.editing_value = "30".to_string();
        app.save_settings();

        assert_eq!(app.data.config.work_duration_secs, 30 * 60);
    }

    #[test]
    fn test_settings_editing_buffer_isolation() {
        let mut app = App::default();
        app.data.config.work_duration_secs = 25 * 60;

        // Simulate opening settings modal
        app.settings.editing_config = app.data.config.clone();
        app.settings.field = SettingsField::WorkDuration;
        app.settings.editing_value = "30".to_string();

        // Apply the editing value (simulates navigation)
        app.apply_editing_value();

        // editing_config should be updated, config should not
        assert_eq!(app.settings.editing_config.work_duration_secs, 30 * 60);
        assert_eq!(app.data.config.work_duration_secs, 25 * 60);
    }

    #[test]
    fn test_settings_save_commits_all_changes() {
        let mut app = App::default();
        app.data.config.work_duration_secs = 25 * 60;
        app.data.config.short_break_secs = 5 * 60;

        // Simulate editing multiple fields
        app.settings.editing_config = app.data.config.clone();

        // Edit work duration
        app.settings.field = SettingsField::WorkDuration;
        app.settings.editing_value = "30".to_string();
        app.apply_editing_value();

        // Navigate to short break and edit it
        app.settings.field = SettingsField::ShortBreak;
        app.settings.editing_value = "10".to_string();
        app.apply_editing_value();

        // Save should commit both changes
        app.settings.field = SettingsField::ShortBreak;
        app.settings.editing_value = "10".to_string();
        app.save_settings();

        assert_eq!(app.data.config.work_duration_secs, 30 * 60);
        assert_eq!(app.data.config.short_break_secs, 10 * 60);
    }

    #[test]
    fn test_settings_cancel_discards_changes() {
        let mut app = App::default();
        app.data.config.work_duration_secs = 25 * 60;

        // Simulate editing
        app.settings.editing_config = app.data.config.clone();
        app.settings.field = SettingsField::WorkDuration;
        app.settings.editing_value = "30".to_string();
        app.apply_editing_value();

        // Cancel (just close modal without saving)
        app.modal = ModalState::None;

        // Config should be unchanged
        assert_eq!(app.data.config.work_duration_secs, 25 * 60);
    }

    #[test]
    fn test_notification_set_and_clear() {
        let mut app = App::default();

        // Initially no notification
        assert!(app.notification.is_none());

        // Set a warning notification
        app.notify(NotificationLevel::Warning, "Test warning");
        assert!(app.notification.is_some());
        let n = app.notification.as_ref().unwrap();
        assert_eq!(n.message, "Test warning");
        assert_eq!(n.level, NotificationLevel::Warning);

        // Set an error notification (replaces previous)
        app.notify(NotificationLevel::Error, "Test error");
        let n = app.notification.as_ref().unwrap();
        assert_eq!(n.message, "Test error");
        assert_eq!(n.level, NotificationLevel::Error);

        // Clear notification (simulates key press)
        app.notification = None;
        assert!(app.notification.is_none());
    }

    #[test]
    fn test_key_press_clears_notification() {
        let mut app = App::default();

        // Set a notification
        app.notify(NotificationLevel::Warning, "Test warning");
        assert!(app.notification.is_some());

        // Simulate a key press - notification should be cleared
        app.handle_key_event(KeyEvent::from(KeyCode::Char('x')));
        assert!(app.notification.is_none());
    }
}
