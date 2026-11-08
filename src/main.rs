use color_eyre::Result;

use app::App;

mod app;
mod listener;
mod process;
mod runtime;
mod snapshot;
mod ui;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app = App::new()?;

    ratatui::run(|terminal| app.run(terminal))
}
