mod connection;
pub mod queries;
mod schema;

pub use connection::Database;
pub use queries::{
    get_categories, get_config, get_sessions_in_range, get_time_by_category, save_config,
    save_session,
};
