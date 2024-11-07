use crate::database::Database;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

#[derive(Clone)]
pub struct SubTask {
    pub id: i64,
    pub title: String,
    pub status: bool,
}

#[derive(Clone)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub due_date: String,
    pub status: bool,
    pub subtasks: Vec<SubTask>,
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
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
    AddingSubtask,
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
        }
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.area());

        let input = Paragraph::new(self.input.as_ref())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Search"));
        f.render_widget(input, chunks[0]);

        let tasks: Vec<ListItem> = self
            .filtered_tasks
            .iter()
            .map(|task| {
                let content = vec![Span::raw(&task.title)];
                ListItem::new(content)
            })
            .collect();

        let tasks = List::new(tasks)
            .block(Block::default().borders(Borders::ALL).title("Tasks"))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            );

        let mut task_state = ListState::default();
        task_state.select(self.selected_task);
        f.render_stateful_widget(tasks, chunks[1], &mut task_state);

        if let Some(selected_task) = self.selected_task {
            let subtasks: Vec<ListItem> = self.filtered_tasks[selected_task]
                .subtasks
                .iter()
                .map(|subtask| {
                    let content = vec![Span::raw(&subtask.title)];
                    ListItem::new(content)
                })
                .collect();

            let subtasks = List::new(subtasks)
                .block(Block::default().borders(Borders::ALL).title("Subtasks"))
                .highlight_style(
                    Style::default()
                        .bg(Color::LightBlue)
                        .add_modifier(Modifier::BOLD),
                );

            let mut subtask_state = ListState::default();
            subtask_state.select(self.selected_subtask);
            f.render_stateful_widget(subtasks, chunks[2], &mut subtask_state);
        }

        let help = Paragraph::new("Press 'q' to quit, 'e' to edit task, 'd' to delete task, 'a' to add subtask, 'Tab' to toggle subtasks, Ctrl+C to mark as complete, Ctrl+A to archive")
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Help"));
        f.render_widget(help, chunks[3]);

        if let Some(dialog) = &self.confirmation_dialog {
            let dialog_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(40),
                        Constraint::Length(3),
                        Constraint::Percentage(40),
                    ]
                    .as_ref(),
                )
                .split(f.area());

            let dialog_content = Paragraph::new(dialog.message.clone())
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Confirmation"));
            f.render_widget(dialog_content, dialog_layout[1]);
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent, db: &Database) -> Result<()> {
        if let Some(dialog) = &self.confirmation_dialog {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    match dialog.action {
                        ConfirmationAction::Quit => return Ok(()),
                        ConfirmationAction::DeleteTask => {
                            if let Some(selected_task) = self.selected_task {
                                let task_id = self.filtered_tasks[selected_task].id;
                                db.delete_task(task_id)?;
                                self.refresh_tasks(db)?;
                            }
                        }
                        ConfirmationAction::DeleteSubtask => {
                            if let (Some(selected_task), Some(selected_subtask)) =
                                (self.selected_task, self.selected_subtask)
                            {
                                let subtask_id = self.filtered_tasks[selected_task].subtasks
                                    [selected_subtask]
                                    .id;
                                db.delete_subtask(subtask_id)?;
                                self.refresh_tasks(db)?;
                            }
                        }
                        ConfirmationAction::ArchiveTask => {
                            if let Some(selected_task) = self.selected_task {
                                let mut task = self.filtered_tasks[selected_task].clone();
                                task.status = true;
                                db.update_task(&task)?;
                                self.refresh_tasks(db)?;
                            }
                        }
                    }
                    self.confirmation_dialog = None;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.confirmation_dialog = None;
                }
                _ => {}
            }
            return Ok(());
        }

        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('e') => {
                    self.input_mode = InputMode::Editing;
                }
                KeyCode::Char('q') => {
                    self.confirmation_dialog = Some(ConfirmationDialog {
                        message: "Are you sure you want to quit? (y/n)".to_string(),
                        action: ConfirmationAction::Quit,
                    });
                }
                KeyCode::Char('a') => {
                    if self.selected_task.is_some() {
                        self.input_mode = InputMode::AddingSubtask;
                    }
                }
                KeyCode::Char('d') => {
                    if let Some(selected_task) = self.selected_task {
                        if self.selected_subtask.is_some() {
                            self.confirmation_dialog = Some(ConfirmationDialog {
                                message: "Are you sure you want to delete this subtask? (y/n)"
                                    .to_string(),
                                action: ConfirmationAction::DeleteSubtask,
                            });
                        } else {
                            self.confirmation_dialog = Some(ConfirmationDialog {
                                message: "Are you sure you want to delete this task? (y/n)"
                                    .to_string(),
                                action: ConfirmationAction::DeleteTask,
                            });
                        }
                    }
                }
                KeyCode::Tab => {
                    if let Some(selected_task) = self.selected_task {
                        if self.filtered_tasks[selected_task].subtasks.is_empty() {
                            self.selected_subtask = None;
                        } else if self.selected_subtask.is_none() {
                            self.selected_subtask = Some(0);
                        } else {
                            self.selected_subtask = None;
                        }
                    }
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(selected_task) = self.selected_task {
                        if let Some(selected_subtask) = self.selected_subtask {
                            let mut subtask = self.filtered_tasks[selected_task].subtasks
                                [selected_subtask]
                                .clone();
                            subtask.status = true;
                            db.update_subtask(&subtask)?;
                        } else {
                            let mut task = self.filtered_tasks[selected_task].clone();
                            task.status = true;
                            db.update_task(&task)?;
                        }
                        self.refresh_tasks(db)?;
                    }
                }
                KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(selected_task) = self.selected_task {
                        self.confirmation_dialog = Some(ConfirmationDialog {
                            message: "Are you sure you want to archive this task? (y/n)"
                                .to_string(),
                            action: ConfirmationAction::ArchiveTask,
                        });
                    }
                }
                KeyCode::Down => {
                    if let Some(selected) = self.selected_task {
                        if selected < self.filtered_tasks.len() - 1 {
                            self.selected_task = Some(selected + 1);
                            self.selected_subtask = None;
                        }
                    } else {
                        self.selected_task = Some(0);
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = self.selected_task {
                        if selected > 0 {
                            self.selected_task = Some(selected - 1);
                            self.selected_subtask = None;
                        }
                    }
                }
                KeyCode::Right => {
                    if let Some(selected_task) = self.selected_task {
                        if self.filtered_tasks[selected_task].subtasks.is_empty() {
                            self.selected_subtask = None;
                        } else {
                            self.selected_subtask = Some(0);
                        }
                    }
                }
                KeyCode::Left => {
                    self.selected_subtask = None;
                }
                _ => {}
            },
            InputMode::Editing => match key.code {
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                    // Add task to database and refresh tasks
                    db.add_task(&self.input)?;
                    self.refresh_tasks(db)?;
                    self.input.clear();
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                    self.filter_tasks();
                }
                KeyCode::Backspace => {
                    self.input.pop();
                    self.filter_tasks();
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input.clear();
                    self.filter_tasks();
                }
                _ => {}
            },
            InputMode::AddingSubtask => match key.code {
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                    if let Some(selected_task) = self.selected_task {
                        let task_id = self.filtered_tasks[selected_task].id;
                        db.add_subtask(task_id, &self.input)?;
                        self.refresh_tasks(db)?;
                    }
                    self.input.clear();
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input.clear();
                }
                _ => {}
            },
        }
        Ok(())
    }

    pub fn on_tick(&self) {
        // Update app state here
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
            self.filtered_tasks = self
                .tasks
                .iter()
                .filter(|task| self.matcher.fuzzy_match(&task.title, &self.input).is_some())
                .cloned()
                .collect();
        }
    }
}
