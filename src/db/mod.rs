use sqlx::{sqlite::SqlitePool, Sqlite, Pool};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::path::PathBuf;

pub mod migrations;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub repo_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub id: i64,
    pub name: String,
    pub order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub column_id: i64,
    pub assignee: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub branch_name: Option<String>,
    pub pr_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub task_id: i64,
    pub author: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idea {
    pub id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: i64,
    pub event: String,
    pub metadata: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(db_path: &PathBuf) -> Result<Self> {
        let db_url = format!("sqlite:{}", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;
        
        Ok(Database { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    // Project operations
    pub async fn create_project(&self, name: &str, repo_path: &str) -> Result<Project> {
        let project = sqlx::query_as!(
            Project,
            "INSERT INTO projects (name, repo_path) VALUES (?, ?) RETURNING *",
            name,
            repo_path
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(project)
    }

    pub async fn get_project_by_path(&self, repo_path: &str) -> Result<Option<Project>> {
        let project = sqlx::query_as!(
            Project,
            "SELECT * FROM projects WHERE repo_path = ?",
            repo_path
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(project)
    }

    // Column operations
    pub async fn create_default_columns(&self) -> Result<Vec<Column>> {
        let default_columns = vec![
            ("Backlog", 0),
            ("To Do", 1),
            ("Doing", 2),
            ("Review", 3),
            ("Done", 4),
        ];

        let mut columns = Vec::new();
        for (name, order) in default_columns {
            let column = sqlx::query_as!(
                Column,
                "INSERT INTO columns (name, \"order\") VALUES (?, ?) RETURNING *",
                name,
                order
            )
            .fetch_one(&self.pool)
            .await?;
            columns.push(column);
        }

        Ok(columns)
    }

    pub async fn get_columns(&self) -> Result<Vec<Column>> {
        let columns = sqlx::query_as!(
            Column,
            "SELECT * FROM columns ORDER BY \"order\""
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(columns)
    }

    pub async fn get_column_by_name(&self, name: &str) -> Result<Option<Column>> {
        let column = sqlx::query_as!(
            Column,
            "SELECT * FROM columns WHERE name = ?",
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(column)
    }

    // Task operations
    pub async fn create_task(&self, title: &str, description: Option<String>, column_id: i64) -> Result<Task> {
        let now = Utc::now();
        let task = sqlx::query_as!(
            Task,
            "INSERT INTO tasks (title, description, column_id, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?) RETURNING *",
            title,
            description,
            column_id,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn get_task(&self, id: i64) -> Result<Option<Task>> {
        let task = sqlx::query_as!(
            Task,
            "SELECT * FROM tasks WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn get_tasks(&self, column_id: Option<i64>) -> Result<Vec<Task>> {
        let tasks = if let Some(column_id) = column_id {
            sqlx::query_as!(
                Task,
                "SELECT * FROM tasks WHERE column_id = ? ORDER BY created_at DESC",
                column_id
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                Task,
                "SELECT * FROM tasks ORDER BY column_id, created_at DESC"
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(tasks)
    }

    pub async fn update_task_column(&self, id: i64, column_id: i64) -> Result<()> {
        let now = Utc::now();
        sqlx::query!(
            "UPDATE tasks SET column_id = ?, updated_at = ? WHERE id = ?",
            column_id,
            now,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_task_branch(&self, id: i64, branch_name: &str) -> Result<()> {
        let now = Utc::now();
        sqlx::query!(
            "UPDATE tasks SET branch_name = ?, updated_at = ? WHERE id = ?",
            branch_name,
            now,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_task_pr(&self, id: i64, pr_url: &str) -> Result<()> {
        let now = Utc::now();
        sqlx::query!(
            "UPDATE tasks SET pr_url = ?, updated_at = ? WHERE id = ?",
            pr_url,
            now,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Comment operations
    pub async fn create_comment(&self, task_id: i64, author: &str, text: &str) -> Result<Comment> {
        let now = Utc::now();
        let comment = sqlx::query_as!(
            Comment,
            "INSERT INTO comments (task_id, author, text, created_at) VALUES (?, ?, ?, ?) RETURNING *",
            task_id,
            author,
            text,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    pub async fn get_comments(&self, task_id: i64) -> Result<Vec<Comment>> {
        let comments = sqlx::query_as!(
            Comment,
            "SELECT * FROM comments WHERE task_id = ? ORDER BY created_at",
            task_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(comments)
    }

    // Idea operations
    pub async fn create_idea(&self, content: &str) -> Result<Idea> {
        let now = Utc::now();
        let idea = sqlx::query_as!(
            Idea,
            "INSERT INTO ideas (content, created_at) VALUES (?, ?) RETURNING *",
            content,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(idea)
    }

    pub async fn get_idea(&self, id: i64) -> Result<Option<Idea>> {
        let idea = sqlx::query_as!(
            Idea,
            "SELECT * FROM ideas WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(idea)
    }

    pub async fn get_ideas(&self) -> Result<Vec<Idea>> {
        let ideas = sqlx::query_as!(
            Idea,
            "SELECT * FROM ideas ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(ideas)
    }

    pub async fn delete_idea(&self, id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM ideas WHERE id = ?", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Activity log operations
    pub async fn log_activity(&self, event: &str, metadata: Option<String>) -> Result<()> {
        let now = Utc::now();
        sqlx::query!(
            "INSERT INTO activity_log (event, metadata, created_at) VALUES (?, ?, ?)",
            event,
            metadata,
            now
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_activity_log(&self, limit: Option<i64>) -> Result<Vec<ActivityLog>> {
        let limit = limit.unwrap_or(50);
        let logs = sqlx::query_as!(
            ActivityLog,
            "SELECT * FROM activity_log ORDER BY created_at DESC LIMIT ?",
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }
}
