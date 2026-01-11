use chrono::{DateTime, Local};

/// A completed pomodoro session
#[derive(Debug, Clone)]
pub struct Session {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub started_at: i64, // Unix timestamp
    pub ended_at: i64,   // Unix timestamp
    pub duration_secs: i64,
}

impl Session {
    /// Get the start time as a DateTime
    pub fn start_datetime(&self) -> DateTime<Local> {
        DateTime::from_timestamp(self.started_at, 0)
            .map(|dt| dt.with_timezone(&Local))
            .unwrap_or_else(Local::now)
    }

    /// Get the end time as a DateTime
    pub fn end_datetime(&self) -> DateTime<Local> {
        DateTime::from_timestamp(self.ended_at, 0)
            .map(|dt| dt.with_timezone(&Local))
            .unwrap_or_else(Local::now)
    }

    /// Format duration as "Xh Ym" or "Xm"
    pub fn format_duration(&self) -> String {
        let minutes = self.duration_secs / 60;
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
