mod app;
mod config;
mod db;
mod event;
mod models;
mod timer;
mod ui;

use app::App;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new()?.run(terminal);
    ratatui::restore();
    result
}
