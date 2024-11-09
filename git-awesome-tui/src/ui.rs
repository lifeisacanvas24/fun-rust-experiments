use crate::data::Category;
use webbrowser;

pub fn start_ui(categories: Vec<Category>) {
    let mut selected_category = 0;
    let mut selected_subcategory = 0;
    let mut selected_link = 0;

    loop {
        // Display categories, subcategories, and links (3 columns)
        display_categories(
            &categories,
            selected_category,
            selected_subcategory,
            selected_link,
        );

        // Handle user input (tab, arrow keys, enter)
        if let Some(input) = handle_input() {
            match input.as_str() {
                // Handle Tab key to navigate columns
                "tab" => {
                    // Logic to switch between columns
                }
                // Handle arrow keys to navigate through categories/subcategories
                "up" => {
                    // Logic to move up through options
                }
                "down" => {
                    // Logic to move down through options
                }
                // Handle Enter or Right arrow key to open a link
                "enter" | "right" => {
                    if let Some(subcategories) = &categories[selected_category].get_subcategories()
                    {
                        if let Some(links) = subcategories[selected_subcategory].get_links() {
                            open_link(&links[selected_link]);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn display_categories(
    _categories: &[Category],
    _selected_category: usize,
    _selected_subcategory: usize,
    _selected_link: usize,
) {
    // Code to display categories in the first column, subcategories in the second column,
    // and links in the third column.
}

pub fn handle_input() -> Option<String> {
    // Code to handle user input, such as Tab, Arrow keys, Enter, etc.
    // Return input like "tab", "up", "down", "enter", etc.
    None
}

pub fn open_link(link: &str) {
    if webbrowser::open(link).is_err() {
        eprintln!("Failed to open URL: {}", link);
    }
}
