use crate::data::{Category, Link, Subcategory};
use crossterm::{cursor, event, execute, terminal};
use ratatui::text::Text;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{Frame, Terminal},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io::{self, stdout};

pub fn start_ui(categories: Vec<Category>) {
    let categories: Vec<Category> = categories
        .into_iter()
        .filter(|category| category.title != "Uncategorized")
        .collect();

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
    let mut current_page = 0;
    let items_per_page = 25;
    let mut quitting = false;
    let search_query = String::new();

    loop {
        terminal
            .draw(|f| {
                let size = f.area();
                let clear_block = Block::default().style(Style::default().bg(Color::Reset));
                f.render_widget(clear_block, size);

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(10), // Search Bar
                        Constraint::Percentage(80), // Main Content
                        Constraint::Percentage(10), // Help Bar
                    ])
                    .split(f.area());

                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(20), // Categories
                        Constraint::Percentage(40), // Subcategories / Links
                        Constraint::Percentage(40), // Links of Sub Categories
                    ])
                    .split(chunks[1]);

                render_search_bar(f, chunks[0], &search_query);
                render_categories(
                    f,
                    main_chunks[0],
                    &categories,
                    selected_category,
                    current_page,
                    items_per_page,
                );
                if let Some(selected_category) = categories.get(selected_category) {
                    if let Some(subcategories) = &selected_category.subcategories {
                        render_subcategories(
                            f,
                            main_chunks[1],
                            subcategories,
                            selected_subcategory,
                        );
                    } else if let Some(direct_links) = &selected_category.direct_links {
                        render_direct_links(f, main_chunks[1], direct_links);
                    }
                }

                render_help_bar(f, chunks[2]);

                if quitting {
                    render_quit_dialog(f, size);
                }
            })
            .unwrap();

        if quitting {
            if let Some(input) = handle_input() {
                match input.as_str() {
                    "y" => break,            // Quit the application
                    "n" => quitting = false, // Cancel quit
                    _ => {}
                }
            }
        } else {
            if let Some(input) = handle_input() {
                match input.as_str() {
                    "up" => {
                        if selected_category > 0 {
                            selected_category -= 1;
                        }
                    }
                    "down" => {
                        if selected_category < categories.len() - 1 {
                            selected_category += 1;
                        }
                    }
                    "left" => {
                        if selected_subcategory > 0 {
                            selected_subcategory -= 1;
                        }
                    }
                    "right" => {
                        if let Some(selected_category) = categories.get(selected_category) {
                            if let Some(subcategories) = &selected_category.subcategories {
                                if selected_subcategory < subcategories.len() - 1 {
                                    selected_subcategory += 1;
                                }
                            }
                        }
                    }
                    "page_up" => {
                        if current_page > 0 {
                            current_page -= 1;
                        }
                    }
                    "page_down" => {
                        if current_page < categories.len() / items_per_page {
                            current_page += 1;
                        }
                    }
                    "esc" => {
                        quitting = true;
                    }
                    "enter" => {
                        // Handle search input here (e.g., filter categories)
                    }
                    _ => {}
                }
            }
        }
    }

    terminal::disable_raw_mode().unwrap();
    execute!(
        io::stdout(),
        terminal::LeaveAlternateScreen,
        terminal::Clear(terminal::ClearType::All),
        cursor::Show
    )
    .unwrap();
}

fn render_search_bar(f: &mut Frame, area: Rect, search_query: &str) {
    let search_text = format!("Search: {}", search_query);
    let paragraph = Paragraph::new(Text::from(search_text))
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(paragraph, area);
}

fn render_categories(
    f: &mut Frame,
    area: Rect,
    categories: &[Category],
    selected: usize,
    page: usize,
    items_per_page: usize,
) {
    let start_index = page * items_per_page;
    let end_index = std::cmp::min(start_index + items_per_page, categories.len());

    let items: Vec<ListItem> = categories[start_index..end_index]
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

fn render_subcategories(f: &mut Frame, area: Rect, subcategories: &[Subcategory], selected: usize) {
    let items: Vec<ListItem> = subcategories
        .iter()
        .enumerate()
        .map(|(i, subcategory)| {
            let content = if i == selected {
                format!("> {}", subcategory.title)
            } else {
                format!("  {}", subcategory.title)
            };
            ListItem::new(content).style(if i == selected {
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

fn render_direct_links(f: &mut Frame, area: Rect, links: &[Link]) {
    let items: Vec<ListItem> = links
        .iter()
        .map(|link| ListItem::new(link.title.clone()))
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Direct Links"));
    f.render_widget(list, area);
}

fn render_help_bar(f: &mut Frame, area: Rect) {
    let help_text = Paragraph::new(
        "Esc - Quit | Up/Down - Browse Categories | Left/Right - Browse Subcategories | Page Up/Page Down - Navigate Pages",
    )
    .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help_text, area);
}

fn render_quit_dialog(f: &mut Frame, size: Rect) {
    let dialog_width = 40;
    let dialog_height = 7;
    let x = (size.width - dialog_width) / 2;
    let y = (size.height - dialog_height) / 2;

    let confirm_msg = Paragraph::new("Are you sure you want to quit? (y/n)")
        .block(Block::default().borders(Borders::ALL).title("Confirm Quit"))
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(confirm_msg, Rect::new(x, y, dialog_width, dialog_height));
}

pub fn handle_input() -> Option<String> {
    if let Ok(event) = event::read() {
        if let event::Event::Key(key_event) = event {
            return match key_event.code {
                event::KeyCode::Up => Some("up".to_string()),
                event::KeyCode::Down => Some("down".to_string()),
                event::KeyCode::Esc => Some("esc".to_string()),
                event::KeyCode::PageUp => Some("page_up".to_string()),
                event::KeyCode::PageDown => Some("page_down".to_string()),
                event::KeyCode::Left => Some("left".to_string()),
                event::KeyCode::Right => Some("right".to_string()),
                event::KeyCode::Char(c) if c == 'y' || c == 'n' => Some(c.to_string()),
                _ => None,
            };
        }
    }
    None
}
