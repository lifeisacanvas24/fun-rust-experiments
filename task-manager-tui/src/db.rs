use rusqlite::{params, Connection, Result};

#[derive(Debug)]
pub struct Task {
    #[allow(dead_code)]
    pub id: i32,
    pub title: String,
    pub description: String,
    pub category: String,
    pub parent_task_id: Option<i32>,
    pub due_date: Option<String>, // You can use a date type if you prefer
    pub time: Option<String>,     // Format: "HH:MM"
    pub priority: String,
    pub status: String, // New field for task status
}

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn new(db_file: &str) -> Result<Self> {
        let connection = Connection::open(db_file)?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                category TEXT,
                parent_task_id INTEGER,
                due_date TEXT,
                time TEXT,
                priority TEXT,
                status TEXT NOT NULL DEFAULT 'To Do' -- Default status
            )",
            [],
        )?;
        Ok(Database { connection })
    }

    pub fn add_task(&self, task: &Task) -> Result<()> {
        self.connection.execute(
            "INSERT INTO tasks (title, description, category, parent_task_id, due_date, time, priority, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                task.title,
                task.description,
                task.category,
                task.parent_task_id,
                task.due_date,
                task.time,
                task.priority,
                task.status, // Include status when adding a task
            ],
        )?;
        Ok(())
    }

    // Additional methods for fetching tasks, etc., can be added here
}
