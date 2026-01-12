use chrono::{DateTime, Local};
use ratatui::style::Color;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

/// A string with a maximum length enforced at runtime.
/// Silently ignores characters that would exceed the limit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BoundedString<const MAX: usize>(String);

impl<const MAX: usize> BoundedString<MAX> {
    /// Create from a string, truncating to fit within MAX bytes.
    /// Truncation respects UTF-8 character boundaries.
    pub fn from_string(s: impl Into<String>) -> Self {
        let s = s.into();
        if s.len() <= MAX {
            BoundedString(s)
        } else {
            // Find the last valid UTF-8 boundary within MAX bytes
            let mut end = MAX;
            while end > 0 && !s.is_char_boundary(end) {
                end -= 1;
            }
            BoundedString(s[..end].to_string())
        }
    }

    pub fn push(&mut self, c: char) {
        if self.0.len() + c.len_utf8() <= MAX {
            self.0.push(c);
        }
    }

    pub fn pop(&mut self) -> Option<char> {
        self.0.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Check if the string is blank (empty or contains only whitespace)
    pub fn is_blank(&self) -> bool {
        self.0.trim().is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl<const MAX: usize> std::fmt::Display for BoundedString<MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Unix timestamp in seconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Timestamp(i64);

impl Timestamp {
    pub fn new(secs: i64) -> Self {
        Timestamp(secs)
    }

    pub fn now() -> Self {
        Timestamp(Local::now().timestamp())
    }

    pub fn to_datetime(self) -> DateTime<Local> {
        DateTime::from_timestamp(self.0, 0)
            .map(|dt| dt.with_timezone(&Local))
            .unwrap_or_else(Local::now)
    }
}

impl From<i64> for Timestamp {
    fn from(val: i64) -> Self {
        Timestamp(val)
    }
}

impl From<Timestamp> for i64 {
    fn from(t: Timestamp) -> Self {
        t.0
    }
}

impl std::ops::Sub for Timestamp {
    type Output = DurationSecs;

    fn sub(self, other: Self) -> DurationSecs {
        DurationSecs(self.0 - other.0)
    }
}

impl ToSql for Timestamp {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for Timestamp {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        i64::column_result(value).map(Timestamp)
    }
}

/// Duration in seconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct DurationSecs(i64);

impl DurationSecs {
    pub fn new(secs: i64) -> Self {
        DurationSecs(secs)
    }

    pub fn as_secs(self) -> i64 {
        self.0
    }
}

impl From<i64> for DurationSecs {
    fn from(val: i64) -> Self {
        DurationSecs(val)
    }
}

impl From<DurationSecs> for i64 {
    fn from(d: DurationSecs) -> Self {
        d.0
    }
}

impl ToSql for DurationSecs {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for DurationSecs {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        i64::column_result(value).map(DurationSecs)
    }
}

/// Database row ID for a session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(i64);

impl SessionId {
    pub fn new(id: i64) -> Self {
        SessionId(id)
    }
}

impl From<i64> for SessionId {
    fn from(val: i64) -> Self {
        SessionId(val)
    }
}

impl From<SessionId> for i64 {
    fn from(id: SessionId) -> Self {
        id.0
    }
}

impl ToSql for SessionId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for SessionId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        i64::column_result(value).map(SessionId)
    }
}

/// Database row ID for a category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CategoryId(i64);

impl From<i64> for CategoryId {
    fn from(val: i64) -> Self {
        CategoryId(val)
    }
}

impl From<CategoryId> for i64 {
    fn from(id: CategoryId) -> Self {
        id.0
    }
}

impl ToSql for CategoryId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for CategoryId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        i64::column_result(value).map(CategoryId)
    }
}

/// A completed pomodoro session
#[derive(Debug, Clone)]
pub struct Session {
    pub id: Option<SessionId>,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub started_at: Timestamp,
    pub ended_at: Timestamp,
    pub duration_secs: DurationSecs,
}

impl Session {
    /// Create a new SessionBuilder
    pub fn builder() -> SessionBuilder {
        SessionBuilder::new()
    }

    /// Get the start time as a DateTime
    pub fn start_datetime(&self) -> DateTime<Local> {
        self.started_at.to_datetime()
    }

    /// Get the end time as a DateTime
    pub fn end_datetime(&self) -> DateTime<Local> {
        self.ended_at.to_datetime()
    }

    /// Format duration as "Xh Ym" or "Xm"
    pub fn format_duration(&self) -> String {
        let minutes = self.duration_secs.as_secs() / 60;
        let hours = minutes / 60;
        let remaining_minutes = minutes % 60;

        if hours > 0 {
            format!("{}h {}m", hours, remaining_minutes)
        } else {
            format!("{}m", minutes)
        }
    }
}

/// Builder for creating Session instances
#[derive(Debug, Default)]
pub struct SessionBuilder {
    name: Option<String>,
    description: Option<String>,
    category: Option<String>,
    started_at: Option<Timestamp>,
    ended_at: Option<Timestamp>,
    duration_secs: Option<DurationSecs>,
}

impl SessionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, desc: Option<String>) -> Self {
        self.description = desc;
        self
    }

    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn started_at(mut self, ts: Timestamp) -> Self {
        self.started_at = Some(ts);
        self
    }

    pub fn ended_at(mut self, ts: Timestamp) -> Self {
        self.ended_at = Some(ts);
        self
    }

    pub fn duration_secs(mut self, d: DurationSecs) -> Self {
        self.duration_secs = Some(d);
        self
    }

    /// Build the Session, returning None if required fields are missing
    pub fn build(self) -> Option<Session> {
        Some(Session {
            id: None,
            name: self.name?,
            description: self.description,
            category: self.category?,
            started_at: self.started_at?,
            ended_at: self.ended_at?,
            duration_secs: self.duration_secs?,
        })
    }
}

/// A category for sessions with an associated color
#[derive(Debug, Clone)]
pub struct Category {
    pub id: Option<CategoryId>,
    pub name: String,
    pub color: Color,
}

impl Category {
    /// Default categories with colors
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                id: None,
                name: "work".to_string(),
                color: Color::Rgb(255, 107, 107), // #FF6B6B
            },
            Self {
                id: None,
                name: "study".to_string(),
                color: Color::Rgb(78, 205, 196), // #4ECDC4
            },
            Self {
                id: None,
                name: "coding".to_string(),
                color: Color::Rgb(69, 183, 209), // #45B7D1
            },
            Self {
                id: None,
                name: "reading".to_string(),
                color: Color::Rgb(150, 206, 180), // #96CEB4
            },
            Self {
                id: None,
                name: "exercise".to_string(),
                color: Color::Rgb(255, 234, 167), // #FFEAA7
            },
            Self {
                id: None,
                name: "other".to_string(),
                color: Color::Rgb(223, 230, 233), // #DFE6E9
            },
        ]
    }
}

/// Parse a hex color string like "#FF6B6B" to a ratatui Color
pub fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return Color::Gray;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
    Color::Rgb(r, g, b)
}

/// Format a ratatui Color as a hex string like "#FF6B6B"
pub fn format_hex_color(color: Color) -> String {
    match color {
        Color::Rgb(r, g, b) => format!("#{:02X}{:02X}{:02X}", r, g, b),
        _ => "#808080".to_string(), // Gray fallback for non-RGB colors
    }
}

/// Aggregated time statistics for a category
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryStat {
    pub name: String,
    pub total_seconds: i64,
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

    /// Check if all config values are valid (positive durations)
    pub fn is_valid(&self) -> bool {
        self.work_duration_secs > 0
            && self.short_break_secs > 0
            && self.long_break_secs > 0
            && self.sessions_until_long_break > 0
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_string_max_length() {
        let mut s: BoundedString<5> = BoundedString::default();
        s.push('a');
        s.push('b');
        s.push('c');
        s.push('d');
        s.push('e');
        s.push('f'); // Should be ignored
        assert_eq!(s.to_string(), "abcde");
    }

    #[test]
    fn test_bounded_string_empty() {
        let s: BoundedString<10> = BoundedString::default();
        assert!(s.is_empty());
        assert_eq!(s.to_string(), "");
    }

    #[test]
    fn test_bounded_string_pop() {
        let mut s: BoundedString<10> = BoundedString::default();
        s.push('a');
        s.push('b');
        assert_eq!(s.pop(), Some('b'));
        assert_eq!(s.to_string(), "a");
    }

    #[test]
    fn test_bounded_string_clear() {
        let mut s: BoundedString<10> = BoundedString::default();
        s.push('a');
        s.push('b');
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn test_bounded_string_utf8_boundary() {
        // Multi-byte character: é is 2 bytes in UTF-8
        let mut s: BoundedString<3> = BoundedString::default();
        s.push('a'); // 1 byte, total 1
        s.push('é'); // 2 bytes, total 3
        s.push('b'); // Would exceed 3 bytes, ignored
        assert_eq!(s.to_string(), "aé");
    }

    #[test]
    fn test_bounded_string_emoji() {
        // Emoji: 🎉 is 4 bytes in UTF-8
        let mut s: BoundedString<4> = BoundedString::default();
        s.push('🎉'); // 4 bytes, fits exactly
        s.push('a'); // Would exceed, ignored
        assert_eq!(s.to_string(), "🎉");
    }

    #[test]
    fn test_bounded_string_whitespace_only() {
        let mut s: BoundedString<10> = BoundedString::default();
        s.push(' ');
        s.push(' ');
        s.push(' ');
        assert!(!s.is_empty()); // is_empty only checks length, not content
        assert!(s.is_blank()); // is_blank checks for whitespace-only
    }

    #[test]
    fn test_bounded_string_is_blank() {
        let empty: BoundedString<10> = BoundedString::default();
        assert!(empty.is_blank());

        let whitespace: BoundedString<10> = BoundedString::from_string("   ");
        assert!(whitespace.is_blank());

        let content: BoundedString<10> = BoundedString::from_string("hello");
        assert!(!content.is_blank());

        let mixed: BoundedString<10> = BoundedString::from_string("  hi  ");
        assert!(!mixed.is_blank());
    }

    #[test]
    fn test_bounded_string_from_string() {
        let s: BoundedString<5> = BoundedString::from_string("abc");
        assert_eq!(s.to_string(), "abc");

        let s: BoundedString<5> = BoundedString::from_string("abcdefgh");
        assert_eq!(s.to_string(), "abcde");
    }

    #[test]
    fn test_bounded_string_from_string_utf8_truncation() {
        // "héllo" - é is 2 bytes, so "héll" is 5 bytes
        let s: BoundedString<5> = BoundedString::from_string("héllo");
        assert_eq!(s.to_string(), "héll");

        // Truncating in the middle of a multi-byte character
        let s: BoundedString<2> = BoundedString::from_string("é"); // é is 2 bytes
        assert_eq!(s.to_string(), "é");

        let s: BoundedString<1> = BoundedString::from_string("é"); // Can't fit, truncates to empty
        assert_eq!(s.to_string(), "");
    }

    #[test]
    fn test_config_default_values() {
        let config = Config::default();
        assert_eq!(config.work_duration_secs, 25 * 60);
        assert_eq!(config.short_break_secs, 5 * 60);
        assert_eq!(config.long_break_secs, 15 * 60);
        assert_eq!(config.sessions_until_long_break, 4);
    }

    #[test]
    fn test_config_constants() {
        assert_eq!(Config::DEFAULT_WORK_SECS, 1500);
        assert_eq!(Config::DEFAULT_SHORT_BREAK_SECS, 300);
        assert_eq!(Config::DEFAULT_LONG_BREAK_SECS, 900);
        assert_eq!(Config::DEFAULT_SESSIONS_UNTIL_LONG, 4);
    }

    #[test]
    fn test_config_custom_values() {
        let config = Config {
            work_duration_secs: 30 * 60,
            short_break_secs: 10 * 60,
            long_break_secs: 20 * 60,
            sessions_until_long_break: 3,
        };
        assert_eq!(config.work_duration_secs, 1800);
        assert_eq!(config.short_break_secs, 600);
        assert_eq!(config.long_break_secs, 1200);
        assert_eq!(config.sessions_until_long_break, 3);
    }

    #[test]
    fn test_config_is_valid() {
        let config = Config::default();
        assert!(config.is_valid());

        let valid = Config {
            work_duration_secs: 1,
            short_break_secs: 1,
            long_break_secs: 1,
            sessions_until_long_break: 1,
        };
        assert!(valid.is_valid());
    }

    #[test]
    fn test_config_is_invalid_zero() {
        let zero_work = Config {
            work_duration_secs: 0,
            ..Config::default()
        };
        assert!(!zero_work.is_valid());

        let zero_short = Config {
            short_break_secs: 0,
            ..Config::default()
        };
        assert!(!zero_short.is_valid());

        let zero_long = Config {
            long_break_secs: 0,
            ..Config::default()
        };
        assert!(!zero_long.is_valid());

        let zero_sessions = Config {
            sessions_until_long_break: 0,
            ..Config::default()
        };
        assert!(!zero_sessions.is_valid());
    }

    #[test]
    fn test_config_is_invalid_negative() {
        let negative = Config {
            work_duration_secs: -1,
            short_break_secs: 5 * 60,
            long_break_secs: 15 * 60,
            sessions_until_long_break: 4,
        };
        assert!(!negative.is_valid());
    }
}
