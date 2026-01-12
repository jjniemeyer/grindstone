mod connection;
pub mod queries;
mod schema;

use ratatui::style::Color;

use crate::error::Result;
use crate::models::{Category, CategoryId, CategoryStat, Config, Session, SessionId};

pub use connection::Database;

/// Trait for database operations, enabling testability via mocking
pub trait DatabaseOps {
    fn save_session(&self, session: &Session) -> Result<SessionId>;
    fn delete_session(&self, id: SessionId) -> Result<usize>;
    fn get_sessions_in_range(&self, start: i64, end: i64) -> Result<Vec<Session>>;
    fn get_time_by_category(&self, start: i64, end: i64) -> Result<Vec<CategoryStat>>;
    fn get_categories(&self) -> Result<Vec<Category>>;
    fn create_category(&self, name: &str, color: Color) -> Result<CategoryId>;
    fn delete_category(&self, id: CategoryId) -> Result<usize>;
    fn update_category(&self, id: CategoryId, name: &str, color: Color) -> Result<usize>;
    fn is_category_in_use(&self, name: &str) -> Result<bool>;
    fn get_config(&self) -> Result<Config>;
    fn save_config(&self, config: &Config) -> Result<()>;
}
