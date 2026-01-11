use rusqlite::Connection;

use crate::config::{
    DEFAULT_LONG_BREAK, DEFAULT_SHORT_BREAK, DEFAULT_WORK_DURATION, SESSIONS_UNTIL_LONG_BREAK,
};
use crate::models::Category;

/// Initialize the database schema
pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            category TEXT NOT NULL DEFAULT 'uncategorized',
            started_at INTEGER NOT NULL,
            ended_at INTEGER NOT NULL,
            duration_secs INTEGER NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions(started_at);
        CREATE INDEX IF NOT EXISTS idx_sessions_category ON sessions(category);

        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL DEFAULT '#808080'
        );

        CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value INTEGER NOT NULL
        );
        ",
    )?;

    // Seed default categories if table is empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;

    if count == 0 {
        let mut stmt =
            conn.prepare("INSERT OR IGNORE INTO categories (name, color) VALUES (?1, ?2)")?;
        for category in Category::defaults() {
            stmt.execute([&category.name, &category.color])?;
        }
    }

    // Seed default config if table is empty
    let config_count: i64 = conn.query_row("SELECT COUNT(*) FROM config", [], |row| row.get(0))?;

    if config_count == 0 {
        let mut stmt = conn.prepare("INSERT INTO config (key, value) VALUES (?1, ?2)")?;
        stmt.execute([
            "work_duration_secs",
            &DEFAULT_WORK_DURATION.as_secs().to_string(),
        ])?;
        stmt.execute([
            "short_break_secs",
            &DEFAULT_SHORT_BREAK.as_secs().to_string(),
        ])?;
        stmt.execute(["long_break_secs", &DEFAULT_LONG_BREAK.as_secs().to_string()])?;
        stmt.execute([
            "sessions_until_long_break",
            &SESSIONS_UNTIL_LONG_BREAK.to_string(),
        ])?;
    }

    Ok(())
}
