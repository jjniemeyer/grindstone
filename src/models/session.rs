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
    /// Create a new session
    pub fn new(
        name: String,
        description: Option<String>,
        category: String,
        started_at: i64,
        ended_at: i64,
    ) -> Self {
        let duration_secs = ended_at - started_at;
        Self {
            id: None,
            name,
            description,
            category,
            started_at,
            ended_at,
            duration_secs,
        }
    }

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

    /// Parse hex color to RGB values
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        let hex = self.color.trim_start_matches('#');
        if hex.len() != 6 {
            return (128, 128, 128); // Default gray
        }

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
        (r, g, b)
    }
}
