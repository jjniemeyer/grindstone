use chrono::{DateTime, Local};

/// Unix timestamp in seconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Timestamp(pub i64);

impl Timestamp {
    pub fn now() -> Self {
        Timestamp(Local::now().timestamp())
    }
}

impl From<i64> for Timestamp {
    fn from(val: i64) -> Self {
        Timestamp(val)
    }
}

impl std::ops::Sub for Timestamp {
    type Output = DurationSecs;

    fn sub(self, other: Self) -> DurationSecs {
        DurationSecs(self.0 - other.0)
    }
}

/// Duration in seconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct DurationSecs(pub i64);

impl From<i64> for DurationSecs {
    fn from(val: i64) -> Self {
        DurationSecs(val)
    }
}

/// A completed pomodoro session
#[derive(Debug, Clone)]
pub struct Session {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub started_at: Timestamp,
    pub ended_at: Timestamp,
    pub duration_secs: DurationSecs,
}

impl Session {
    /// Get the start time as a DateTime
    pub fn start_datetime(&self) -> DateTime<Local> {
        DateTime::from_timestamp(self.started_at.0, 0)
            .map(|dt| dt.with_timezone(&Local))
            .unwrap_or_else(Local::now)
    }

    /// Get the end time as a DateTime
    pub fn end_datetime(&self) -> DateTime<Local> {
        DateTime::from_timestamp(self.ended_at.0, 0)
            .map(|dt| dt.with_timezone(&Local))
            .unwrap_or_else(Local::now)
    }

    /// Format duration as "Xh Ym" or "Xm"
    pub fn format_duration(&self) -> String {
        let minutes = self.duration_secs.0 / 60;
        let hours = minutes / 60;
        let remaining_minutes = minutes % 60;

        if hours > 0 {
            format!("{}h {}m", hours, remaining_minutes)
        } else {
            format!("{}m", minutes)
        }
    }
}

/// A category for sessions with an associated color
#[derive(Debug, Clone)]
pub struct Category {
    #[allow(dead_code)]
    pub id: Option<i64>,
    pub name: String,
    pub color: String, // Hex color like "#FF6B6B"
}

impl Category {
    /// Default categories with colors
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                id: None,
                name: "work".to_string(),
                color: "#FF6B6B".to_string(),
            },
            Self {
                id: None,
                name: "study".to_string(),
                color: "#4ECDC4".to_string(),
            },
            Self {
                id: None,
                name: "coding".to_string(),
                color: "#45B7D1".to_string(),
            },
            Self {
                id: None,
                name: "reading".to_string(),
                color: "#96CEB4".to_string(),
            },
            Self {
                id: None,
                name: "exercise".to_string(),
                color: "#FFEAA7".to_string(),
            },
            Self {
                id: None,
                name: "other".to_string(),
                color: "#DFE6E9".to_string(),
            },
        ]
    }
}

/// Timer configuration settings
#[derive(Debug, Clone)]
pub struct Config {
    pub work_duration_secs: i64,
    pub short_break_secs: i64,
    pub long_break_secs: i64,
    pub sessions_until_long_break: i64,
}

impl Config {
    pub const DEFAULT_WORK_SECS: i64 = 25 * 60;
    pub const DEFAULT_SHORT_BREAK_SECS: i64 = 5 * 60;
    pub const DEFAULT_LONG_BREAK_SECS: i64 = 15 * 60;
    pub const DEFAULT_SESSIONS_UNTIL_LONG: i64 = 4;
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_duration_secs: Self::DEFAULT_WORK_SECS,
            short_break_secs: Self::DEFAULT_SHORT_BREAK_SECS,
            long_break_secs: Self::DEFAULT_LONG_BREAK_SECS,
            sessions_until_long_break: Self::DEFAULT_SESSIONS_UNTIL_LONG,
        }
    }
}
