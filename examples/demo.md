# ProjectBoard CLI Demo

This demo walks through the basic workflow of using ProjectBoard CLI.

## Setup

```bash
# Initialize git repository (if not already done)
git init
git remote add origin https://github.com/username/repo.git

# Initialize ProjectBoard
pb init
```

## Basic Workflow

```bash
# Add some tasks
pb add "Implement user authentication" --description "Add login/logout functionality"
pb add "Create dashboard UI"
pb add "Add unit tests"

# Add some ideas for later
pb idea "Add dark mode theme"
pb idea "Implement real-time notifications"

# View all tasks
pb list

# Start working on first task
pb start 1
# This creates branch: feature/1-implement-user-authentication
# Moves task to "Doing" column

# Make your changes...
# git add .

# Complete the task
pb done 1 --message "Add basic auth with JWT tokens"
# Commits changes, pushes branch, moves to "Done"

# Submit for review
pb submit 1
# Creates GitHub PR, moves to "Review"

# Work on another task
pb start 2

# View the board
pb board
# Opens interactive TUI board

# Export for reporting
pb export --csv > tasks.csv
pb export --markdown > board.md
```

## Advanced Usage

```bash
# Move tasks between columns manually
pb move 2 "To Do"

# Add comments to tasks
pb comment 2 "Need to decide on UI framework first"

# Promote ideas to tasks
pb promote 1  # Converts idea #1 to a task in Backlog

# Review PR status
pb review 1
```
