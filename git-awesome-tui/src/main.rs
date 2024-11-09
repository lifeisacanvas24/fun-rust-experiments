mod data;
mod ui;

use data::read_awesome_json;

fn main() {
    // Load the categories from awesome.json
    let categories = match read_awesome_json() {
        Ok(categories) => categories,
        Err(e) => {
            eprintln!("Failed to load awesome.json: {}", e);
            return;
        }
    };

    // Start the UI rendering and input handling
    ui::start_ui(categories);
}
