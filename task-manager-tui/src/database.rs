use crate::app::{SubTask, Task};
use color_eyre::Result;
use rusqlite::{params, Connection};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("tasks.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                due_date TEXT,
                status INTEGER
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS subtasks (
                id INTEGER PRIMARY KEY,
                task_id INTEGER,
                title TEXT NOT NULL,
                status INTEGER,
                FOREIGN KEY(task_id) REFERENCES tasks(id)
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn add_task(&self, title: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO tasks (title, status) VALUES (?1, 0)",
            params![title],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn add_subtask(&self, task_id: i64, title: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO subtasks (task_id, title, status) VALUES (?1, ?2, 0)",
            params![task_id, title],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_tasks(&self) -> Result<Vec<Task>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, description, due_date, status FROM tasks")?;
        let task_iter = stmt.query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                due_date: row.get(3)?,
                status: row.get(4)?,
                subtasks: Vec::new(),
            })
        })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            let mut task = task?;
            task.subtasks = self.get_subtasks(task.id)?;
            tasks.push(task);
        }
        Ok(tasks)
    }

    pub fn get_subtasks(&self, task_id: i64) -> Result<Vec<SubTask>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, status FROM subtasks WHERE task_id = ?1")?;
        let subtask_iter = stmt.query_map(params![task_id], |row| {
            Ok(SubTask {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
            })
        })?;

        let mut subtasks = Vec::new();
        for subtask in subtask_iter {
            subtasks.push(subtask?);
        }
        Ok(subtasks)
    }

    pub fn update_task(&self, task: &Task) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET title = ?1, description = ?2, due_date = ?3, status = ?4 WHERE id = ?5",
            params![task.title, task.description, task.due_date, task.status, task.id],
        )?;
        Ok(())
    }

    pub fn update_subtask(&self, subtask: &SubTask) -> Result<()> {
        self.conn.execute(
            "UPDATE subtasks SET title = ?1, status = ?2 WHERE id = ?3",
            params![subtask.title, subtask.status, subtask.id],
        )?;
        Ok(())
    }

    pub fn delete_task(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM subtasks WHERE task_id = ?1", params![id])?;
        self.conn
            .execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn delete_subtask(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM subtasks WHERE id = ?1", params![id])?;
        Ok(())
    }
}
