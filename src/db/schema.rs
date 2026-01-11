use rusqlite::Connection;

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

    Ok(())
}
