mod connection;
pub mod queries;
mod schema;

pub use connection::Database;
pub use queries::{
    create_category, delete_category, delete_session, get_categories, get_config,
    get_sessions_in_range, get_time_by_category, is_category_in_use, save_config, save_session,
};
