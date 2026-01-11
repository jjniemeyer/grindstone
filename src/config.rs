use directories::ProjectDirs;
use std::path::PathBuf;
use std::time::Duration;

/// Default work duration (25 minutes)
pub const DEFAULT_WORK_DURATION: Duration = Duration::from_secs(25 * 60);

/// Default short break duration (5 minutes)
pub const DEFAULT_SHORT_BREAK: Duration = Duration::from_secs(5 * 60);

/// Default long break duration (15 minutes)
pub const DEFAULT_LONG_BREAK: Duration = Duration::from_secs(15 * 60);

/// Number of work sessions before a long break
pub const SESSIONS_UNTIL_LONG_BREAK: u8 = 4;

/// Tick rate for the event loop (100ms)
pub const TICK_RATE: Duration = Duration::from_millis(100);

/// Get the path to the database file.
///
/// Returns the path to `grindstone.db` in the appropriate data directory:
/// - Linux: `~/.local/share/grindstone/grindstone.db`
/// - macOS: `~/Library/Application Support/grindstone/grindstone.db`
/// - Windows: `C:\Users\<User>\AppData\Roaming\grindstone\grindstone.db`
pub fn get_db_path() -> color_eyre::Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", "grindstone")
        .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine data directory"))?;

    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;

    Ok(data_dir.join("grindstone.db"))
}
