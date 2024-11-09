use crate::data::{Category, Subcategory};
use crossterm::{cursor, event, execute, terminal, ExecutableCommand};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{Frame, Terminal},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io::{self, stdout, Write};

pub fn start_ui(categories: Vec<Category>) {
    // Configure the terminal for raw mode and alternate screen
    terminal::enable_raw_mode().unwrap();
    let mut stdout = stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide
    )
    .unwrap();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut selected_category = 0;
    let mut selected_subcategory = 0;
    let mut selected_link = 0;
    let mut quitting = false;

    loop {
        terminal.draw(|f| {
            // Clear the terminal background to avoid seeing previous frame artifacts
            let size = f.size();
            let clear_block = Block::default().style(Style::default().bg(Color::Reset));
            f.render_widget(clear_block, size);

            // Layout configuration
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(10), // Search Bar
                    Constraint::Percentage(80), // Main Content
                    Constraint::Percentage(10), // Help Bar
                ])
                .split(f.size());

            // Render Search Bar
            let search_bar = Paragraph::new("Search: (Not Implemented)")
                .block(Block::default().borders(Borders::ALL).title("Search"));
            f.render_widget(search_bar, chunks[0]);

            // Main Content Layout
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33), // Categories
                    Constraint::Percentage(33), // Subcategories
                    Constraint::Percentage(34), // Links
                ])
                .split(chunks[1]);

            render_categories(f, main_chunks[0], &categories, selected_category);
            render_subcategories(
                f,
                main_chunks[1],
                &categories,
                selected_category,
                selected_subcategory,
            );
            render_links(
                f,
                main_chunks[2],
                &categories,
                selected_category,
                selected_subcategory,
                selected_link,
            );

            // Render Help Bar
            let help_text = Paragraph::new(
                "Esc - Quit | Tab - View Categories/Links | Enter or Right Arrow - Open Link | Up/Down - Browse Items",
            )
            .block(Block::default().borders(Borders::ALL).title("Help"));
            f.render_widget(help_text, chunks[2]);

            // Show confirmation dialog if quitting
            if quitting {
                let dialog_width = 40;
                let dialog_height = 7;
                let x = (size.width - dialog_width) / 2;
                let y = (size.height - dialog_height) / 2;

                let confirm_msg = Paragraph::new("Are you sure you want to quit? (y/n)")
                    .block(Block::default().borders(Borders::ALL).title("Confirm Quit"))
                    .style(Style::default().fg(Color::Yellow));

                f.render_widget(confirm_msg, Rect::new(x, y, dialog_width, dialog_height));
            }
        })
        .unwrap();

        // Handle user input
        if quitting {
            if let Some(input) = handle_input() {
                match input.as_str() {
                    "y" => break,            // Quit the application
                    "n" => quitting = false, // Cancel the quit
                    _ => {}
                }
            }
        } else {
            if let Some(input) = handle_input() {
                match input.as_str() {
                    "tab" => {
                        // Logic to switch between columns
                    }
                    "up" => {
                        // Move up logic with boundary checks
                        if selected_category > 0 {
                            selected_category -= 1;
                        }
                    }
                    "down" => {
                        // Move down logic with boundary checks
                        if selected_category < categories.len() - 1 {
                            selected_category += 1;
                        }
                    }
                    "enter" | "right" => {
                        // Open link
                    }
                    "esc" => {
                        quitting = true; // Trigger the confirmation dialog
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal to its normal state
    terminal::disable_raw_mode().unwrap();
    execute!(
        io::stdout(),
        terminal::LeaveAlternateScreen,
        terminal::Clear(terminal::ClearType::All),
        cursor::Show
    )
    .unwrap();
}

fn render_categories(f: &mut Frame, area: Rect, categories: &[Category], selected: usize) {
    let items: Vec<ListItem> = categories
        .iter()
        .enumerate()
        .map(|(i, category)| {
            let content = if i == selected {
                format!("> {}", category.title)
            } else {
                format!("  {}", category.title)
            };
            ListItem::new(content).style(if i == selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            })
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Categories"));
    f.render_widget(list, area);
}

fn render_subcategories(
    f: &mut Frame,
    area: Rect,
    categories: &[Category],
    selected_category: usize,
    selected_subcategory: usize,
) {
    if let Some(subcategories) = categories.get(selected_category).map(|c| &c.subcategories) {
        let items: Vec<ListItem> = subcategories
            .iter()
            .enumerate()
            .map(|(i, subcategory)| {
                let content = if i == selected_subcategory {
                    format!("> {}", subcategory.title)
                } else {
                    format!("  {}", subcategory.title)
                };
                ListItem::new(content).style(if i == selected_subcategory {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                })
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Subcategories"),
        );
        f.render_widget(list, area);
    }
}

fn render_links(
    f: &mut Frame,
    area: Rect,
    categories: &[Category],
    selected_category: usize,
    selected_subcategory: usize,
    selected_link: usize,
) {
    if let Some(links) = categories
        .get(selected_category)
        .and_then(|category| category.subcategories.get(selected_subcategory))
        .map(|subcategory| &subcategory.links)
    {
        let items: Vec<ListItem> = links
            .iter()
            .enumerate()
            .map(|(i, link)| {
                let content = if i == selected_link {
                    format!("> {}", link.title)
                } else {
                    format!("  {}", link.title)
                };
                ListItem::new(content).style(if i == selected_link {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                })
            })
            .collect();

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Links"));
        f.render_widget(list, area);
    }
}

pub fn handle_input() -> Option<String> {
    if let Ok(event) = event::read() {
        if let event::Event::Key(key_event) = event {
            return match key_event.code {
                event::KeyCode::Tab => Some("tab".to_string()),
                event::KeyCode::Up => Some("up".to_string()),
                event::KeyCode::Down => Some("down".to_string()),
                event::KeyCode::Enter => Some("enter".to_string()),
                event::KeyCode::Right => Some("right".to_string()),
                event::KeyCode::Esc => Some("esc".to_string()),
                event::KeyCode::Char(c) if c == 'y' || c == 'n' => Some(c.to_string()),
                _ => None,
            };
        }
    }
    None
}
