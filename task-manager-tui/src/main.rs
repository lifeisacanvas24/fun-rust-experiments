mod db; // Ensure the db module is included

use crossterm::{
    event::{self, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

#[derive(Clone, Copy)]
enum FocusedField {
    Title,
    Description,
    Category,
    ParentTask,
    DueDate,
    Time,
    Priority,
    Status,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::Clear(ClearType::All))?;

    // Create a terminal
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize the database
    let db = db::Database::new("tasks.db")?;

    // Main event loop
    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Define the layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10), // Row 1: Search (10%)
                        Constraint::Percentage(80), // Row 2: Tasks (80%)
                        Constraint::Percentage(10), // Row 3: Help (10%)
                    ]
                    .as_ref(),
                )
                .split(size);

            // Row 1: Search bar
            let search_block = Block::default().title("Search").borders(Borders::ALL);
            f.render_widget(search_block, chunks[0]);

            // Row 2: Task list (split into 2 columns)
            let task_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(50), // Column 1: Main Tasks (50%)
                        Constraint::Percentage(50), // Column 2: Sub-tasks (50%)
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);

            let task_block = Block::default().title("Tasks").borders(Borders::ALL);
            f.render_widget(task_block, task_chunks[0]); // Main Tasks

            let subtask_block = Block::default().title("Sub-tasks").borders(Borders::ALL);
            f.render_widget(subtask_block, task_chunks[1]); // Sub-tasks

            // Row 3: Help section with hotkeys
            let help_text = "Esc to quit - Ctrl + A (Add Task) - Ctrl + E (Edit Task) - Ctrl + C (Mark as Completed) - Ctrl + D (Delete Task) - Tab (View Sub Tasks)";
            let help_block = Paragraph::new(help_text)
                .block(Block::default().title("Help").borders(Borders::ALL));
            f.render_widget(help_block, chunks[2]);
        })?;

        // Handle user input
        if let Ok(true) = event::poll(std::time::Duration::from_millis(100)) {
            if let Ok(event::Event::Key(KeyEvent {
                code, modifiers, ..
            })) = event::read()
            {
                match code {
                    KeyCode::Char('a') if modifiers == event::KeyModifiers::CONTROL => {
                        show_task_dialog(&db)?;
                    }
                    KeyCode::Esc => {
                        terminal::disable_raw_mode()?;
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }
}

fn show_task_dialog(db: &db::Database) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut title = String::new();
    let mut description = String::new();
    let categories = vec!["Personal", "Work", "Development", "Home", "Career Related"];
    let selected_category = categories[0].to_string();
    let priorities = vec!["High", "Medium", "Low"];
    let selected_priority = priorities[0].to_string();
    let statuses = vec!["To Do", "In Progress", "Completed"];
    let selected_status = statuses[0].to_string();
    let mut due_date = String::new();
    let mut time = String::new();
    let parent_task_id: Option<i32> = None;

    // Fetch existing tasks for parent task selection
    let existing_tasks: Vec<db::Task> = Vec::new(); // TODO: Implement get_all_tasks
    let mut filtered_tasks = existing_tasks.iter().collect::<Vec<_>>(); // Collect references instead of cloning
    let search_query = String::new();

    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    let mut focused_field = FocusedField::Title; // Start with the Title field focused

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Draw the dialog
            let dialog_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(10), // Title
                        Constraint::Percentage(10), // Description
                        Constraint::Percentage(10), // Category
                        Constraint::Percentage(10), // Parent Task
                        Constraint::Percentage(10), // Due Date
                        Constraint::Percentage(10), // Time
                        Constraint::Percentage(10), // Priority
                        Constraint::Percentage(10), // Status
                        Constraint::Percentage(10), // Submit
                    ]
                    .as_ref(),
                )
                .split(size);

            // Title input
            let title_block = Block::default().title("Title").borders(Borders::ALL);
            f.render_widget(title_block, dialog_chunks[0]);
            f.render_widget(Paragraph::new(title.as_str()), dialog_chunks[0]);

            // Description input
            let description_block = Block::default().title("Description").borders(Borders::ALL);
            f.render_widget(description_block, dialog_chunks[1]);
            f.render_widget(Paragraph::new(description.as_str()), dialog_chunks[1]);

            // Category selection
            let category_block = Block::default().title("Category").borders(Borders::ALL);
            f.render_widget(category_block.clone(), dialog_chunks[2]);
            let category_list: Vec<ListItem> =
                categories.iter().map(|cat| ListItem::new(*cat)).collect();
            let category_list_widget = List::new(category_list).block(category_block);
            f.render_widget(category_list_widget, dialog_chunks[2]);

            // Parent Task selection with fuzzy search
            let parent_task_block = Block::default().title("Parent Task").borders(Borders::ALL);
            f.render_widget(parent_task_block.clone(), dialog_chunks[3]);
            filtered_tasks = existing_tasks
                .iter()
                .filter(|task| task.title.contains(&search_query))
                .collect();
            let parent_task_list: Vec<ListItem> = filtered_tasks
                .iter()
                .map(|task| ListItem::new(task.title.as_str()))
                .collect();
            let parent_task_list_widget = List::new(parent_task_list).block(parent_task_block);
            f.render_widget(parent_task_list_widget, dialog_chunks[3]);

            // Due Date input
            let due_date_block = Block::default().title("Due Date").borders(Borders::ALL);
            f.render_widget(due_date_block, dialog_chunks[4]);
            f.render_widget(Paragraph::new(due_date.as_str()), dialog_chunks[4]);

            // Time input
            let time_block = Block::default().title("Time").borders(Borders::ALL);
            f.render_widget(time_block, dialog_chunks[5]);
            f.render_widget(Paragraph::new(time.as_str()), dialog_chunks[5]);

            // Priority selection
            let priority_block = Block::default().title("Priority").borders(Borders::ALL);
            f.render_widget(priority_block.clone(), dialog_chunks[6]);
            let priority_list: Vec<ListItem> =
                priorities.iter().map(|pri| ListItem::new(*pri)).collect();
            let priority_list_widget = List::new(priority_list).block(priority_block);
            f.render_widget(priority_list_widget, dialog_chunks[6]);

            // Status selection
            let status_block = Block::default().title("Status").borders(Borders::ALL);
            f.render_widget(status_block.clone(), dialog_chunks[7]);
            let status_list: Vec<ListItem> =
                statuses.iter().map(|stat| ListItem::new(*stat)).collect();
            let status_list_widget = List::new(status_list).block(status_block);
            f.render_widget(status_list_widget, dialog_chunks[7]);

            // Submit button
            let submit_block = Block::default().title("Submit").borders(Borders::ALL);
            f.render_widget(submit_block, dialog_chunks[8]);
        })?;

        // Handle input for dialog fields
        if let Ok(true) = event::poll(std::time::Duration::from_millis(100)) {
            if let Ok(event::Event::Key(KeyEvent {
                code, modifiers: _, ..
            })) = event::read()
            {
                match code {
                    KeyCode::Enter => {
                        // Create a new task and add it to the database
                        let new_task = db::Task {
                            id: 0, // This will be auto-incremented
                            title: title.clone(),
                            description: description.clone(),
                            category: selected_category.clone(),
                            parent_task_id, // Set parent task ID if selected
                            due_date: Some(due_date.clone()),
                            time: Some(time.clone()),
                            priority: selected_priority.clone(),
                            status: selected_status.clone(),
                        };
                        db.add_task(&new_task)?;
                        break; // Exit the dialog
                    }
                    KeyCode::Esc => break, // Exit the dialog
                    KeyCode::Tab => {
                        // Switch focus to the next field
                        focused_field = match focused_field {
                            FocusedField::Title => FocusedField::Description,
                            FocusedField::Description => FocusedField::Category,
                            FocusedField::Category => FocusedField::ParentTask,
                            FocusedField::ParentTask => FocusedField::DueDate,
                            FocusedField::DueDate => FocusedField::Time,
                            FocusedField::Time => FocusedField::Priority,
                            FocusedField::Priority => FocusedField::Status,
                            FocusedField::Status => FocusedField::Title,
                        };
                    }
                    KeyCode::BackTab => {
                        // Switch focus to the previous field
                        focused_field = match focused_field {
                            FocusedField::Title => FocusedField::Status,
                            FocusedField::Description => FocusedField::Title,
                            FocusedField::Category => FocusedField::Description,
                            FocusedField::ParentTask => FocusedField::Category,
                            FocusedField::DueDate => FocusedField::ParentTask,
                            FocusedField::Time => FocusedField::DueDate,
                            FocusedField::Priority => FocusedField::Time,
                            FocusedField::Status => FocusedField::Priority,
                        };
                    }
                    KeyCode::Char(c) => {
                        // Update the input fields based on the current context
                        match focused_field {
                            FocusedField::Title => title.push(c),
                            FocusedField::Description => description.push(c),
                            FocusedField::DueDate => due_date.push(c),
                            FocusedField::Time => time.push(c),
                            _ => {}
                        }
                    }
                    KeyCode::Backspace => {
                        // Handle backspace for the active input field
                        match focused_field {
                            FocusedField::Title => {
                                title.pop();
                            }
                            FocusedField::Description => {
                                description.pop();
                            }
                            FocusedField::DueDate => {
                                due_date.pop();
                            }
                            FocusedField::Time => {
                                time.pop();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
