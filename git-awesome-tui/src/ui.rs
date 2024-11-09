use crate::data::{Category, Subcategory};
use crossterm::{cursor, event, execute, terminal, ExecutableCommand};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{Frame, Terminal},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io::{stdout, Write};

pub fn start_ui(categories: Vec<Category>) {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    let mut selected_category = 0;
    let mut selected_subcategory = 0;
    let mut selected_link = 0;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(10), // Search Bar
                    Constraint::Percentage(80), // Main Content
                    Constraint::Percentage(10), // Help Bar
                ])
                .split(f.size());

            // Search Bar
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

            // Help Bar
            let help_text = Paragraph::new(
                "Esc - Quit | Tab - View Categories/Links | Enter or Right Arrow - Open Link | Up/Down - Browse Items",
            )
            .block(Block::default().borders(Borders::ALL).title("Help"));
            f.render_widget(help_text, chunks[2]);
        }).unwrap();

        if let Some(input) = handle_input() {
            match input.as_str() {
                "tab" => {
                    // Logic to switch between columns
                }
                "up" => {
                    // Move up logic with boundary checks
                }
                "down" => {
                    // Move down logic with boundary checks
                }
                "enter" | "right" => {
                    // Open link
                }
                "esc" => break,
                _ => {}
            }
        }
    }
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

// Clears the terminal screen and moves the cursor to the top-left corner.
fn clear_terminal() {
    let mut stdout = stdout();
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();
}

// Opens a URL in the default web browser.
pub fn open_link(link: &str) {
    if webbrowser::open(link).is_err() {
        eprintln!("Failed to open URL: {}", link);
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
                _ => None,
            };
        }
    }
    None
}
