use ratatui::style::Color;
use rusqlite::Connection;

use crate::config::get_db_path;
use crate::error::Result;
use crate::models::{Category, CategoryId, CategoryStat, Config, Session, SessionId};

use super::schema::init_schema;
use super::{DatabaseOps, queries};

/// Database connection wrapper
pub struct Database {
    pub conn: Connection,
}

impl Database {
    /// Open the database, creating it if necessary
    pub fn open() -> Result<Self> {
        let path = get_db_path()?;
        let conn = Connection::open(&path)?;
        init_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Open an in-memory database (for testing)
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;
        Ok(Self { conn })
    }
}

impl DatabaseOps for Database {
    fn save_session(&self, session: &Session) -> Result<SessionId> {
        queries::save_session(&self.conn, session).map_err(Into::into)
    }

    fn delete_session(&self, id: SessionId) -> Result<usize> {
        queries::delete_session(&self.conn, id).map_err(Into::into)
    }

    fn get_sessions_in_range(&self, start: i64, end: i64) -> Result<Vec<Session>> {
        queries::get_sessions_in_range(&self.conn, start, end).map_err(Into::into)
    }

    fn get_time_by_category(&self, start: i64, end: i64) -> Result<Vec<CategoryStat>> {
        queries::get_time_by_category(&self.conn, start, end).map_err(Into::into)
    }

    fn get_categories(&self) -> Result<Vec<Category>> {
        queries::get_categories(&self.conn).map_err(Into::into)
    }

    fn create_category(&self, name: &str, color: Color) -> Result<CategoryId> {
        queries::create_category(&self.conn, name, color).map_err(Into::into)
    }

    fn delete_category(&self, id: CategoryId) -> Result<usize> {
        queries::delete_category(&self.conn, id).map_err(Into::into)
    }

    fn update_category(&self, id: CategoryId, name: &str, color: Color) -> Result<usize> {
        queries::update_category(&self.conn, id, name, color).map_err(Into::into)
    }

    fn is_category_in_use(&self, name: &str) -> Result<bool> {
        queries::is_category_in_use(&self.conn, name).map_err(Into::into)
    }

    fn get_config(&self) -> Result<Config> {
        queries::get_config(&self.conn).map_err(Into::into)
    }

    fn save_config(&self, config: &Config) -> Result<()> {
        queries::save_config(&self.conn, config).map_err(Into::into)
    }
}
