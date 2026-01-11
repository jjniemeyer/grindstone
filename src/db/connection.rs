use rusqlite::Connection;

use crate::config::get_db_path;

use super::schema::init_schema;

/// Database connection wrapper
pub struct Database {
    pub conn: Connection,
}

impl Database {
    /// Open the database, creating it if necessary
    pub fn open() -> color_eyre::Result<Self> {
        let path = get_db_path()?;
        let conn = Connection::open(&path)?;
        init_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Open an in-memory database (for testing)
    #[cfg(test)]
    pub fn open_in_memory() -> color_eyre::Result<Self> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;
        Ok(Self { conn })
    }
}
