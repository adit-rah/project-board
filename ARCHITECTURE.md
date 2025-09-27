# ProjectBoard CLI Architecture

## Overview

ProjectBoard CLI is a terminal-first project management tool that integrates deeply with git workflows. It provides a Kanban-style board interface while maintaining local-first data storage and seamless git operations.

## Core Components

### 1. CLI Interface (`src/main.rs`)
- Built with `clap` for command parsing
- Async runtime with `tokio`
- Command routing to appropriate handlers

### 2. Database Layer (`src/db/`)
- SQLite storage via `sqlx`
- Schema migrations in `migrations/`
- Core entities: Projects, Columns, Tasks, Comments, Ideas, ActivityLog

### 3. Git Integration (`src/git/`)
- `git2-rs` for git operations
- Branch creation and checkout
- Commit and push operations
- Repository state detection

### 4. GitHub Integration (`src/github/`)
- REST API integration for PR creation
- Repository URL parsing
- Authentication via GITHUB_TOKEN

### 5. TUI Interface (`src/tui/`)
- `ratatui` for terminal UI
- Interactive Kanban board
- Keyboard navigation

### 6. Commands (`src/commands/`)
- All CLI command implementations
- Business logic coordination
- Activity logging

## Data Flow

```
CLI Command → Command Handler → Database + Git/GitHub APIs → Response
```

### Example: `pb start <id>`

1. Parse command arguments
2. Load task from database
3. Create git branch with naming convention
4. Checkout new branch
5. Update task with branch name
6. Move task to "Doing" column
7. Log activity
8. Display success message

## File Structure

```
.projectboard/
├── board.sqlite          # Local SQLite database
└── config.toml          # Future: repo-specific config

src/
├── main.rs              # CLI entry point
├── lib.rs               # Library interface
├── commands/mod.rs      # Command implementations
├── db/mod.rs            # Database layer
├── git/mod.rs           # Git operations
├── github/mod.rs        # GitHub API integration
└── tui/mod.rs           # Terminal UI

migrations/
└── 001_initial.sql      # Database schema

tests/
└── integration_test.rs  # Integration tests
```

## Key Design Principles

1. **Local-First**: All data stored locally in SQLite
2. **Git-Native**: Integrates with existing git workflows
3. **Terminal-First**: CLI-centric with optional TUI
4. **Single Binary**: Self-contained Rust binary
5. **Extensible**: Modular architecture for future features

## Configuration

### Environment Variables
- `GITHUB_TOKEN`: For GitHub API authentication
- `RUST_LOG`: For debug logging

### Future Configuration
- `.projectboard/config.toml`: Repo-specific settings
- Default branch names
- Column customization
- GitHub repository mapping

## Dependencies

### Core
- `clap`: CLI argument parsing
- `tokio`: Async runtime
- `sqlx`: Database operations
- `git2`: Git integration
- `anyhow`: Error handling

### UI
- `ratatui`: Terminal UI
- `crossterm`: Terminal control

### External APIs
- `reqwest`: HTTP client for GitHub API
- `serde`: JSON serialization

## Future Extensions

1. **Multi-Repository Support**: Manage multiple projects
2. **Team Collaboration**: Sync board state across team
3. **CI/CD Integration**: Auto-update based on pipeline status
4. **Custom Workflows**: Configurable column layouts
5. **Plugin System**: Custom command extensions
