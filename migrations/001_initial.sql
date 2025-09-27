-- Initial schema for ProjectBoard CLI

-- Projects table
CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    repo_path TEXT NOT NULL UNIQUE
);

-- Columns table (Backlog, To Do, Doing, Review, Done)
CREATE TABLE columns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    "order" INTEGER NOT NULL
);

-- Tasks table
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    column_id INTEGER NOT NULL,
    assignee TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    branch_name TEXT,
    pr_url TEXT,
    FOREIGN KEY (column_id) REFERENCES columns (id)
);

-- Comments table
CREATE TABLE comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    author TEXT NOT NULL,
    text TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks (id) ON DELETE CASCADE
);

-- Ideas table (brainstorm ideas)
CREATE TABLE ideas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Activity log table
CREATE TABLE activity_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event TEXT NOT NULL,
    metadata TEXT,
    created_at TEXT NOT NULL
);

-- Create indexes for better performance
CREATE INDEX idx_tasks_column_id ON tasks(column_id);
CREATE INDEX idx_tasks_created_at ON tasks(created_at);
CREATE INDEX idx_comments_task_id ON comments(task_id);
CREATE INDEX idx_activity_log_created_at ON activity_log(created_at);
