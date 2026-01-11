mod connection;
pub mod queries;
mod schema;

pub use connection::Database;
pub use queries::{
    delete_session, get_categories, get_sessions_in_range, get_time_by_category, save_session,
};
