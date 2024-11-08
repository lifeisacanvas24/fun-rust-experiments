
use chrono::{NaiveDate, Local};
use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, Paragraph, ListState, Clear},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::time::Duration;
use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Read, Write};
use crate::database::Database;
use crate::config::{Config, StorageType};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub id: i64,
    pub title: String,
    pub status: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub due_date: Option<NaiveDate>,
    pub status: bool,
    pub priority: Priority,
    pub subtasks: Vec<SubTask>,
}

#[derive(Clone, PartialEq)]
pub enum SortMode {
    Default,
    Title,
    DueDate,
    Priority,
    Status,
}

pub struct App {
    tasks: Vec<Task>,
    filtered_tasks: Vec<Task>,
    selected_task: Option<usize>,
    selected_subtask: Option<usize>,
    input_mode: InputMode,
    input: String,
    matcher: SkimMatcherV2,
    confirmation_dialog: Option<ConfirmationDialog>,
    sort_mode: SortMode,
    error_message: Option<String>,
    show_confirm_quit: bool,
    show_help: bool,
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
    AddingSubtask,
    EditingSubtask,
    SettingDueDate,
    SettingPriority,
}

struct ConfirmationDialog {
    message: String,
    action: ConfirmationAction,
}

enum ConfirmationAction {
    Quit,
    DeleteTask,
    DeleteSubtask,
    ArchiveTask,
}

impl App {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            filtered_tasks: Vec::new(),
            selected_task: None,
            selected_subtask: None,
            input_mode: InputMode::Normal,
            input: String::new(),
            matcher: SkimMatcherV2::default(),
            confirmation_dialog: None,
            sort_mode: SortMode::Default,
            error_message: None,
            show_confirm_quit: false,
            show_help: false,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent, db: &Database) -> Result<()> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode(key, db),
            InputMode::Editing => self.handle_editing_mode(key, db),
            InputMode::AddingSubtask => self.handle_adding_subtask_mode(key, db),
            InputMode::EditingSubtask => self.handle_editing_subtask_mode(key, db),
            InputMode::SettingDueDate => self.handle_setting_due_date_mode(key, db),
            InputMode::SettingPriority => self.handle_setting_priority_mode(key, db),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent, db: &Database) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => {
                self.show_confirm_quit = true;
            }
            KeyCode::Char('e') => {
                if let Some(selected_task) = self.selected_task {
                    self.input = self.filtered_tasks[selected_task].title.clone();
                    self.input_mode = InputMode::Editing;
                }
            }
            KeyCode::Char('a') => {
                self.input.clear();
                self.input_mode = InputMode::Editing;
            }
            KeyCode::Char('s') => {
                self.cycle_sort_mode();
                self.sort_tasks();
            }
            KeyCode::Char('d') => {
                if let Some(selected_task) = self.selected_task {
                    self.confirmation_dialog = Some(ConfirmationDialog {
                        message: "Are you sure you want to delete this task? (y/n)".to_string(),
                        action: ConfirmationAction::DeleteTask,
                    });
                }
            }
            KeyCode::Down => {
                if let Some(selected) = self.selected_task {
                    if selected < self.filtered_tasks.len() - 1 {
                        self.selected_task = Some(selected + 1);
                    }
                } else if !self.filtered_tasks.is_empty() {
                    self.selected_task = Some(0);
                }
            }
            KeyCode::Up => {
                if let Some(selected) = self.selected_task {
                    if selected > 0 {
                        self.selected_task = Some(selected - 1);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_editing_mode(&mut self, key: KeyEvent, db: &Database) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                if let Some(selected_task) = self.selected_task {
                    let mut task = self.filtered_tasks[selected_task].clone();
                    task.title = self.input.clone();
                    db.update_task(&task)?;
                } else {
                    let new_task = Task {
                        id: 0,
                        title: self.input.clone(),
                        description: String::new(),
                        due_date: None,
                        status: false,
                        priority: Priority::Medium,
                        subtasks: Vec::new(),
                    };
                    db.add_task(&new_task)?;
                }
                self.refresh_tasks(db)?;
                self.input.clear();
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Esc => {
                self.input.clear();
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_adding_subtask_mode(&mut self, key: KeyEvent, db: &Database) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                if let Some(selected_task) = self.selected_task {
                    let task_id = self.filtered_tasks[selected_task].id;
                    let new_subtask = SubTask {
                        id: 0,
                        title: self.input.clone(),
                        status: false,
                    };
                    db.add_subtask(task_id, &new_subtask)?;
                    self.refresh_tasks(db)?;
                }
                self.input.clear();
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Esc => {
                self.input.clear();
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_editing_subtask_mode(&mut self, key: KeyEvent, db: &Database) -> Result<()> {
        // Implementation for editing subtask
        Ok(())
    }

    fn handle_setting_due_date_mode(&mut self, key: KeyEvent, db: &Database) -> Result<()> {
        // Implementation for setting due date
        Ok(())
    }

    fn handle_setting_priority_mode(&mut self, key: KeyEvent, db: &Database) -> Result<()> {
        // Implementation for setting priority
        Ok(())
    }

    pub fn render(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.size());

        let input = Paragraph::new(self.input.as_ref())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input, chunks[0]);

        let tasks: Vec<ListItem> = self
            .filtered_tasks
            .iter()
            .map(|task| {
                let content = Line::from(vec![
                    Span::raw(if task.status { "[x] " } else { "[ ] " }),
                    Span::styled(&task.title, Style::default().fg(Color::White)),
                ]);
                ListItem::new(content)
            })
            .collect();

        let tasks = List::new(tasks)
            .block(Block::default().borders(Borders::ALL).title("Tasks"))
            .highlight_style(Style::default().bg(Color::DarkGray));

        let mut state = ListState::default();
        state.select(self.selected_task);
        f.render_stateful_widget(tasks, chunks[1], &mut state);

        let help_text = "Press 'q' to quit, 'e' to edit, 'a' to add, 'd' to delete, 's' to sort";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Help"));
        f.render_widget(help, chunks[2]);

        if let Some(error) = &self.error_message {
            let error_message = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red))
                .block(Block::default().borders(Borders::ALL).title("Error"));
            let area = centered_rect(60, 20, f.size());
            f.render_widget(Clear, area);
            f.render_widget(error_message, area);
        }

        if self.show_confirm_quit {
            let confirm_message = Paragraph::new("Are you sure you want to quit? (y/n)")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Confirm"));
            let area = centered_rect(60, 20, f.size());
            f.render_widget(Clear, area);
            f.render_widget(confirm_message, area);
        }
    }

    pub fn refresh_tasks(&mut self, db: &Database) -> Result<()> {
        self.tasks = db.get_tasks()?;
        self.filter_tasks();
        Ok(())
    }

    fn filter_tasks(&mut self) {
        if self.input.is_empty() {
            self.filtered_tasks = self.tasks.clone();
        } else {
            self.filtered_tasks = self.tasks
                .iter()
                .filter(|task| {
                    self.matcher.fuzzy_match(&task.title, &self.input).is_some()
                })
                .cloned()
                .collect();
        }
        self.sort_tasks();
    }

    fn sort_tasks(&mut self) {
        match self.sort_mode {
            SortMode::Default => {}
            SortMode::Title => self.filtered_tasks.sort_by(|a, b| a.title.cmp(&b.title)),
            SortMode::DueDate => self.filtered_tasks.sort_by(|a, b| a.due_date.cmp(&b.due_date)),
            SortMode::Priority => self.filtered_tasks.sort_by(|a, b| b.priority.cmp(&a.priority)),
            SortMode::Status => self.filtered_tasks.sort_by(|a, b| b.status.cmp(&a.status)),
        }
    }

    fn cycle_sort_mode(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Default => SortMode::Title,
            SortMode::Title => SortMode::DueDate,
            SortMode::DueDate => SortMode::Priority,
            SortMode::Priority => SortMode::Status,
            SortMode::Status => SortMode::Default,
        };
    }

    pub fn show_error(&mut self, message: String) {
        self.error_message = Some(message);
    }

    pub fn should_quit(&self) -> bool {
        self.show_confirm_quit
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
