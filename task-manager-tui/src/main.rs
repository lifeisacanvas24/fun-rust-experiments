use color_eyre::Result;
use std::panic;
mod tui;
mod app;
mod database;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up panic hook for better error reporting
    panic::set_hook(Box::new(|panic_info| {
        if let Some(location) = panic_info.location() {
            eprintln!("Panic occurred in file '{}' at line {}", location.file(), location.line());
        }
        eprintln!("Error: {}", panic_info);
    }));

    // Install color_eyre for better error reporting
    color_eyre::install()?;

    let mut tui = tui::Tui::new()?;
    let app = app::App::new();
    let db = database::Database::new()?;

    tui.enter()?;
    let result = tui.run(app, db).await;
    tui.exit()?;

    // Handle any errors that occurred during execution
    if let Err(err) = result {
        eprintln!("An error occurred: {:?}", err);
        Err(err)
    } else {
        Ok(())
    }
}
