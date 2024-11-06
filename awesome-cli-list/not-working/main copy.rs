use comrak::{nodes::NodeValue, parse_document, Arena, ComrakOptions};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{self},
};
use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use reqwest::Client;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::{error::Error, io::StdoutLock};

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
    is_confirming_quit: bool,
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
            is_confirming_quit: false,
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

        for node in root.descendants() {
            if let NodeValue::Link(link) = &node.data.borrow().value {
                let url = if link.url.starts_with('#') {
                    format!("{}/{}", BASE_URL, link.url)
                } else {
                    link.url.to_string()
                };
                let mut link_text = String::new();
                let mut sub_links = Vec::new();

                for child in node.children() {
                    if let NodeValue::Text(text) = &child.data.borrow().value {
                        link_text = text.clone();
                    } else if let NodeValue::List(_) = &child.data.borrow().value {
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
                                        }
                                    }
                                    if !sub_link_text.is_empty() {
                                        Some((sub_link_text, sub_url))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }
                }
                if !link_text.is_empty() {
                    items.push((link_text, url, sub_links));
                }
            }
        }

        items
    }

    fn draw(&mut self, f: &mut Frame<CrosstermBackend<StdoutLock>>, area: Rect) {
        let search_paragraph = Paragraph::new(self.search_display.clone())
            .block(Block::default().title("Search").borders(Borders::ALL));
        f.render_widget(search_paragraph, Rect::new(area.x, area.y, area.width, 3));

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

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Links (Press Enter to open)")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::White));

        f.render_stateful_widget(
            list,
            Rect::new(area.x, area.y + 4, area.width, area.height - 4),
            &mut self.state,
        );

        if self.is_confirming_quit {
            let confirmation_paragraph = Paragraph::new("Are you sure you want to quit? (y/n)")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Quit Confirmation"),
                )
                .alignment(Alignment::Center);
            f.render_widget(
                confirmation_paragraph,
                Rect::new(area.x, area.y + area.height / 2, area.width, 3),
            );
        }
    }

    fn handle_input(&mut self, input: KeyEvent) {
        // Handle quit confirmation
        if self.is_confirming_quit {
            match input.code {
                KeyCode::Char('y') => {
                    std::process::exit(0);
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.is_confirming_quit = false;
                }
                _ => {}
            }
            return; // Skip further processing if in quit confirmation
        }

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
                        let _ = open_in_browser(url);
                    }
                } else if let Some((_, url, _)) = &self.filtered_items.get(self.selected) {
                    let _ = open_in_browser(url);
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
                self.is_confirming_quit = true;
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

fn open_in_browser(url: &str) -> Result<(), Box<dyn Error>> {
    #[cfg(target_os = "linux")]
    Command::new("xdg-open").arg(url).spawn()?;

    #[cfg(target_os = "macos")]
    Command::new("open").arg(url).spawn()?;

    #[cfg(target_os = "windows")]
    Command::new("cmd").args(&["/C", "start", url]).spawn()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    terminal::enable_raw_mode()?;

    // Removed mutability from stdout since it's not required
    let stdout = io::stdout().lock();

    // Make `app` mutable
    let mut app = App::new().await?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            app.draw(f, size);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key_event) = event::read()? {
                app.handle_input(key_event);
            }
        }
    }
}
