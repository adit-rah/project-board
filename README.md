# pb (project-board) CLI

A terminal-first project board tool that wraps around git workflows, enabling developers to manage tasks, branches, and pull requests seamlessly while keeping a project board structure (columns, cards, brainstorm notes).

## Features

- **Local-first**: SQLite database stored in `.projectboard/` within your repo
- **Git Integration**: Automatic branch creation, commits, and push operations
- **GitHub Integration**: Automatic PR creation and status tracking
- **Kanban Board**: Organize tasks across columns (Backlog → To Do → Doing → Review → Done)
- **Ideas Management**: Capture brainstorm ideas and promote them to tasks
- **Export**: CSV and Markdown export for reporting

## Installation

### Prerequisites

Make sure you have Rust installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build from Source

```bash
git clone https://github.com/your-org/projectboard-cli
cd projectboard-cli
cargo build --release
cp target/release/pb /usr/local/bin/  # or add to PATH
```

## Quick Start

1. **Initialize in your git repository**:
   ```bash
   cd your-project
   pb init
   ```

2. **Add your first task**:
   ```bash
   pb add "Implement user authentication"
   ```

3. **Start working on it**:
   ```bash
   pb start 1  # Creates branch, moves to "Doing"
   ```

4. **Complete the task**:
   ```bash
   # Make your changes, stage them with git add
   pb done 1  # Commits, pushes, moves to "Done"
   ```

5. **Submit for review**:
   ```bash
   pb submit 1  # Creates GitHub PR, moves to "Review"
   ```

## Commands

### Task Management
- `pb add "Task title" [--description "Details"]` - Create new task in Backlog
- `pb list [column]` - List tasks (all or by column)
- `pb move <id> <column>` - Move task to different column
- `pb comment <id> "Comment text"` - Add comment to task

### Ideas & Planning
- `pb idea "Add dark mode"` - Capture brainstorm idea
- `pb promote <idea-id>` - Convert idea to task in Backlog

### Git Workflow
- `pb start <id>` - Create branch `feature/<id>-<slug>`, move to Doing
- `pb done <id> [--message "Commit msg"]` - Mark complete, commit, push
- `pb submit <id>` - Push branch, create GitHub PR, move to Review
- `pb review <id>` - Check PR status

### Viewing & Export
- `pb board` - Interactive Kanban board (TUI)
- `pb export --csv` - Export tasks as CSV
- `pb export --markdown` - Export as Markdown

## Configuration

Set your GitHub token for PR creation:
```bash
export GITHUB_TOKEN=your_token_here
```

## Architecture

- **Language**: Rust
- **Database**: SQLite via sqlx
- **Git**: git2-rs (libgit2 bindings)
- **GitHub**: REST API integration
- **TUI**: ratatui for interactive board view

## Development

```bash
# Run tests
cargo test

# Run with debugging
RUST_LOG=debug cargo run -- init

# Build for release
cargo build --release
```

## Roadmap

- [x] Core task management
- [x] Git workflow integration
- [x] GitHub PR creation
- [ ] Interactive TUI board
- [ ] Multi-repository support
- [ ] GitLab/Bitbucket support
- [ ] Team sync via backend service

## License

MIT License - see LICENSE file for details.