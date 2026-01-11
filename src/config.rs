use directories::ProjectDirs;
use std::path::PathBuf;
use std::time::Duration;

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
