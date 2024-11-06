use comrak::{nodes::NodeValue, parse_document, Arena, ComrakOptions};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use ratatui::{prelude::*, widgets::ListState};
use reqwest::Client;
use std::fs;
use std::io;
use std::path::Path;
use std::{error::Error, io::StdoutLock};
use webbrowser;

const README_URL: &str = "https://raw.githubusercontent.com/sindresorhus/awesome/master/readme.md";
const BASE_URL: &str = "https://github.com/sindresorhus/awesome";

struct App {
    items: Vec<(String, String, Vec<(String, String)>)>,
    filtered_items: Vec<(String, String, Vec<(String, String)>)>,
    selected: usize,
    state: ListState,
    sub_links_visible: bool,
    sub_link_selected: usize,
    search_term: String,
    search_display: String,
    show_quit_dialog: bool,
}

impl App {
    async fn new() -> Result<Self, Box<dyn Error>> {
        let content = Self::fetch_and_cache_readme(README_URL).await?;
        let items = Self::parse_readme(&content);
        let mut state = ListState::default();
        state.select(Some(0));

        Ok(Self {
            items: items.clone(),
            filtered_items: items,
            selected: 0,
            state,
            sub_links_visible: false,
            sub_link_selected: 0,
            search_term: String::new(),
            search_display: String::new(),
            show_quit_dialog: false,
        })
    }

    async fn fetch_and_cache_readme(url: &str) -> Result<String, Box<dyn Error>> {
        let client = Client::new();
        let response = client.get(url).send().await?;
        let content = response.text().await?;
        fs::write(Path::new("awesome.md"), &content)?;
        Ok(content)
    }

    fn parse_readme(content: &str) -> Vec<(String, String, Vec<(String, String)>)> {
        let arena = Arena::new();
        let options = ComrakOptions::default();
        let root = parse_document(&arena, content, &options);
        let mut items = Vec::new();

        // Create a file for dumping parse information
        let mut debug_file =
            std::fs::File::create("parse_debug.txt").expect("Failed to create debug file");
        use std::io::Write;

        writeln!(debug_file, "=== Parsing Debug Information ===\n").unwrap();

        for node in root.descendants() {
            if let NodeValue::Link(link) = &node.data.borrow().value {
                let url = if link.url.starts_with('#') {
                    format!("{}/{}", BASE_URL, link.url)
                } else {
                    link.url.to_string()
                };
                let mut link_text = String::new();
                let mut sub_links = Vec::new();

                writeln!(debug_file, "\nMain Link URL: {}", url).unwrap();

                for child in node.children() {
                    if let NodeValue::Text(text) = &child.data.borrow().value {
                        link_text = text.clone();
                        writeln!(debug_file, "Main Link Text: {}", link_text).unwrap();
                    } else if let NodeValue::List(_) = &child.data.borrow().value {
                        writeln!(debug_file, "Found Sub-links:").unwrap();

                        sub_links = child
                            .children()
                            .filter_map(|sub_child| {
                                if let NodeValue::Link(sub_link) = &sub_child.data.borrow().value {
                                    let sub_url = if sub_link.url.starts_with('#') {
                                        format!("{}/{}", BASE_URL, sub_link.url)
                                    } else {
                                        sub_link.url.to_string()
                                    };
                                    let mut sub_link_text = String::new();

                                    for sub_sub_child in sub_child.children() {
                                        if let NodeValue::Text(text) =
                                            &sub_sub_child.data.borrow().value
                                        {
                                            sub_link_text = text.clone();
                                            writeln!(debug_file, "  - Text: {}", sub_link_text)
                                                .unwrap();
                                            writeln!(debug_file, "    URL: {}", sub_url).unwrap();
                                        }
                                    }

                                    if !sub_link_text.is_empty() {
                                        Some((sub_link_text, sub_url))
                                    } else {
                                        None
                                    }
                                } else {
                                    writeln!(debug_file, "  - Found non-link node in sub-links")
                                        .unwrap();
                                    None
                                }
                            })
                            .collect();
                    }
                }

                if !link_text.is_empty() {
                    writeln!(
                        debug_file,
                        "Adding to items: {} (with {} sub-links)\n",
                        link_text,
                        sub_links.len()
                    )
                    .unwrap();
                    items.push((link_text, url, sub_links));
                }
            }
        }

        writeln!(debug_file, "\nTotal items parsed: {}", items.len()).unwrap();
        items
    }
    fn draw(&mut self, f: &mut Frame<CrosstermBackend<StdoutLock>>, area: Rect) {
        // Create the layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(3), // Search bar
                Constraint::Min(0),    // List
            ])
            .split(area);
        // Clear the frame by filling it with an empty block
        let clear_block = Block::default().style(Style::default().bg(Color::Reset));
        f.render_widget(clear_block, area);

        // Render search paragraph
        let search_paragraph = Paragraph::new(self.search_display.clone())
            .block(Block::default().title("Search").borders(Borders::ALL));
        f.render_widget(search_paragraph, Rect::new(area.x, area.y, area.width, 3));

        // Create list items
        let items: Vec<ListItem> = self
            .filtered_items
            .iter()
            .enumerate()
            .map(|(index, (title, _, sub_links))| {
                let mut spans = vec![Span::styled(
                    format!("{}. {}", index + 1, title),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::UNDERLINED),
                )];

                if self.sub_links_visible && self.selected == index {
                    for (sub_index, (sub_title, _)) in sub_links.iter().enumerate() {
                        let style = if self.sub_link_selected == sub_index {
                            Style::default().bg(Color::LightBlue).fg(Color::White)
                        } else {
                            Style::default()
                        };
                        spans.push(Span::styled(
                            format!("\n  - {}. {}", sub_index + 1, sub_title),
                            style,
                        ));
                    }
                }
                ListItem::new(Text::from(Line::from(spans)))
            })
            .collect();

        // Create and render list
        let list = List::new(items)
            .block(
                Block::default()
                    .title("Links (Press Enter to open)")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::White));

        f.render_stateful_widget(list, chunks[1], &mut self.state);

        // Draw quit confirmation dialog if needed
        if self.show_quit_dialog {
            let dialog_width = 50;
            let dialog_height = 3;
            let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
            let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

            let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

            let dialog = Paragraph::new("Press 'y' to quit, 'n' to cancel")
                .block(Block::default().title("Quit?").borders(Borders::ALL))
                .alignment(Alignment::Center);

            f.render_widget(Clear, dialog_area); // Clear the background
            f.render_widget(dialog, dialog_area);
        }
    }

    fn handle_input(&mut self, input: KeyEvent) {
        match input.code {
            KeyCode::Down => {
                if self.sub_links_visible {
                    if self.sub_link_selected < self.filtered_items[self.selected].2.len() - 1 {
                        self.sub_link_selected += 1;
                    }
                } else if self.selected < self.filtered_items.len() - 1 {
                    self.selected += 1;
                    self.state.select(Some(self.selected));
                    self.sub_link_selected = 0;
                    self.sub_links_visible = !self.filtered_items[self.selected].2.is_empty();
                }
            }
            KeyCode::Up => {
                if self.sub_links_visible {
                    if self.sub_link_selected > 0 {
                        self.sub_link_selected -= 1;
                    }
                } else if self.selected > 0 {
                    self.selected -= 1;
                    self.state.select(Some(self.selected));
                    self.sub_link_selected = 0;
                    self.sub_links_visible = !self.filtered_items[self.selected].2.is_empty();
                }
            }
            KeyCode::Enter | KeyCode::Right => {
                if self.sub_links_visible {
                    if let Some((_, url)) = self.filtered_items[self.selected]
                        .2
                        .get(self.sub_link_selected)
                    {
                        let _ = webbrowser::open(url).is_err();
                    }
                } else if let Some((_, url, _)) = &self.filtered_items.get(self.selected) {
                    let _ = webbrowser::open(url).is_err();
                }
            }
            KeyCode::Tab => {
                self.sub_links_visible = !self.sub_links_visible;
                if self.sub_links_visible {
                    self.sub_link_selected = 0;
                }
            }
            KeyCode::Char(c) => {
                self.search_term.push(c);
                self.search_display.push(c);
                self.update_filtered_items();
            }
            KeyCode::Backspace => {
                if self.search_term.pop().is_some() {
                    self.search_display.pop();
                    self.update_filtered_items();
                }
            }
            KeyCode::Esc => {
                if !self.show_quit_dialog {
                    self.show_quit_dialog = true;
                }
            }
            _ => {}
        }
    }

    fn update_filtered_items(&mut self) {
        self.filtered_items = self
            .items
            .iter()
            .filter(|(title, _, _)| {
                title
                    .to_lowercase()
                    .contains(&self.search_term.to_lowercase())
            })
            .cloned()
            .collect();
        if self.selected >= self.filtered_items.len() {
            self.selected = 0;
        }
        self.state.select(Some(self.selected));
    }
}

#[tokio::main]
// Setup terminal
async fn main() -> Result<(), Box<dyn Error>> {
    terminal::enable_raw_mode()?;

    // Clear screen and configure terminal
    execute!(
        io::stdout(),
        terminal::EnterAlternateScreen,
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide
    )?;

    let stdout = io::stdout().lock();
    let mut app = App::new().await?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            app.draw(f, size);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key_event) = event::read()? {
                if app.show_quit_dialog {
                    match key_event.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            break; // Just break the loop
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            app.show_quit_dialog = false;
                        }
                        _ => {} // Ignore other keys when quit dialog is shown
                    }
                } else {
                    // Normal key handling
                    app.handle_input(key_event);
                }
            }
        }
    }
    // Cleanup when the program exits
    terminal::disable_raw_mode()?; // Restore terminal to normal mode
    execute!(
        io::stdout(),
        terminal::LeaveAlternateScreen,
        terminal::Clear(terminal::ClearType::All),
        cursor::Show
    )?;

    Ok(())
}
