use rusqlite::{Connection, params};

use crate::models::{Category, Config, Session};

/// Save a session to the database
pub fn save_session(conn: &Connection, session: &Session) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO sessions (name, description, category, started_at, ended_at, duration_secs)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            session.name,
            session.description,
            session.category,
            session.started_at,
            session.ended_at,
            session.duration_secs,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Get sessions within a time range
pub fn get_sessions_in_range(
    conn: &Connection,
    start: i64,
    end: i64,
) -> rusqlite::Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, category, started_at, ended_at, duration_secs
         FROM sessions
         WHERE started_at >= ?1 AND started_at < ?2
         ORDER BY started_at DESC",
    )?;

    let sessions = stmt.query_map(params![start, end], |row| {
        Ok(Session {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            description: row.get(2)?,
            category: row.get(3)?,
            started_at: row.get(4)?,
            ended_at: row.get(5)?,
            duration_secs: row.get(6)?,
        })
    })?;

    sessions.collect()
}

/// Get total time by category within a time range
pub fn get_time_by_category(
    conn: &Connection,
    start: i64,
    end: i64,
) -> rusqlite::Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT category, SUM(duration_secs) as total
         FROM sessions
         WHERE started_at >= ?1 AND started_at < ?2
         GROUP BY category
         ORDER BY total DESC",
    )?;

    let results = stmt.query_map(params![start, end], |row| Ok((row.get(0)?, row.get(1)?)))?;

    results.collect()
}

/// Get all categories
pub fn get_categories(conn: &Connection) -> rusqlite::Result<Vec<Category>> {
    let mut stmt = conn.prepare("SELECT id, name, color FROM categories ORDER BY name")?;

    let categories = stmt.query_map([], |row| {
        Ok(Category {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            color: row.get(2)?,
        })
    })?;

    categories.collect()
}

/// Delete a session by ID
pub fn delete_session(conn: &Connection, id: i64) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])
}

/// Get timer configuration from database
pub fn get_config(conn: &Connection) -> rusqlite::Result<Config> {
    let mut config = Config::default();

    let mut stmt = conn.prepare("SELECT key, value FROM config")?;
    let rows = stmt.query_map([], |row| {
        let key: String = row.get(0)?;
        let value: i64 = row.get(1)?;
        Ok((key, value))
    })?;

    for row in rows {
        let (key, value) = row?;
        match key.as_str() {
            "work_duration_secs" => config.work_duration_secs = value,
            "short_break_secs" => config.short_break_secs = value,
            "long_break_secs" => config.long_break_secs = value,
            "sessions_until_long_break" => config.sessions_until_long_break = value,
            _ => {}
        }
    }

    Ok(config)
}

/// Save timer configuration to database
pub fn save_config(conn: &Connection, config: &Config) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare("INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)")?;

    stmt.execute(params!["work_duration_secs", config.work_duration_secs])?;
    stmt.execute(params!["short_break_secs", config.short_break_secs])?;
    stmt.execute(params!["long_break_secs", config.long_break_secs])?;
    stmt.execute(params![
        "sessions_until_long_break",
        config.sessions_until_long_break
    ])?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn test_save_and_load_session() {
        let db = Database::open_in_memory().unwrap();
        let session = Session {
            id: None,
            name: "Test session".to_string(),
            description: Some("Description".to_string()),
            category: "coding".to_string(),
            started_at: 1000,
            ended_at: 2500,
            duration_secs: 1500,
        };

        let id = save_session(&db.conn, &session).unwrap();
        assert!(id > 0);

        let sessions = get_sessions_in_range(&db.conn, 0, 3000).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "Test session");
        assert_eq!(sessions[0].duration_secs, 1500);
    }

    #[test]
    fn test_time_by_category() {
        let db = Database::open_in_memory().unwrap();

        let s1 = Session {
            id: None,
            name: "Work 1".to_string(),
            description: None,
            category: "coding".to_string(),
            started_at: 1000,
            ended_at: 2000,
            duration_secs: 1000,
        };
        let s2 = Session {
            id: None,
            name: "Work 2".to_string(),
            description: None,
            category: "coding".to_string(),
            started_at: 2000,
            ended_at: 3000,
            duration_secs: 1000,
        };
        let s3 = Session {
            id: None,
            name: "Meeting".to_string(),
            description: None,
            category: "work".to_string(),
            started_at: 3000,
            ended_at: 4000,
            duration_secs: 1000,
        };

        save_session(&db.conn, &s1).unwrap();
        save_session(&db.conn, &s2).unwrap();
        save_session(&db.conn, &s3).unwrap();

        let totals = get_time_by_category(&db.conn, 0, 5000).unwrap();
        assert_eq!(totals.len(), 2);
        // coding: 2000 seconds, work: 1000 seconds
        assert_eq!(totals[0], ("coding".to_string(), 2000));
        assert_eq!(totals[1], ("work".to_string(), 1000));
    }

    #[test]
    fn test_default_categories_seeded() {
        let db = Database::open_in_memory().unwrap();
        let categories = get_categories(&db.conn).unwrap();
        assert!(!categories.is_empty());
        assert!(categories.iter().any(|c| c.name == "coding"));
    }

    #[test]
    fn test_config_save_and_load() {
        let db = Database::open_in_memory().unwrap();

        // Default config should be seeded
        let config = get_config(&db.conn).unwrap();
        assert_eq!(config.work_duration_secs, 25 * 60);
        assert_eq!(config.short_break_secs, 5 * 60);

        // Modify and save
        let mut new_config = config;
        new_config.work_duration_secs = 30 * 60;
        save_config(&db.conn, &new_config).unwrap();

        // Reload and verify
        let loaded = get_config(&db.conn).unwrap();
        assert_eq!(loaded.work_duration_secs, 30 * 60);
    }
}
