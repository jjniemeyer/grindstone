mod app;
mod config;
mod db;
mod error;
mod event;
mod models;
mod timer;
mod ui;

use app::App;
use log::LevelFilter;
use simplelog::{Config as LogConfig, WriteLogger};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Initialize file logging (ignore errors - logging is optional)
    if let Ok(log_path) = config::get_log_path()
        && let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
    {
        let _ = WriteLogger::init(LevelFilter::Info, LogConfig::default(), file);
    }

    let terminal = ratatui::init();
    let result = App::new()?.run(terminal);
    ratatui::restore();
    result
}
