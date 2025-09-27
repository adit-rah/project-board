use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;
mod db;
mod git;
mod github;
mod tui;

use commands::*;

#[derive(Parser)]
#[command(name = "pb")]
#[command(about = "A terminal-first project board tool that wraps around git workflows")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project board in the current repository
    Init,
    /// Add a new task to the backlog
    Add {
        /// Task title
        title: String,
        /// Optional task description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List tasks, optionally filtered by column
    List {
        /// Column name to filter by
        column: Option<String>,
    },
    /// Move a task to a different column
    Move {
        /// Task ID
        id: u32,
        /// Target column name
        column: String,
    },
    /// Add a comment to a task
    Comment {
        /// Task ID
        id: u32,
        /// Comment text
        text: String,
    },
    /// Add a brainstorm idea
    Idea {
        /// Idea content
        content: String,
    },
    /// Promote an idea to a task in the backlog
    Promote {
        /// Idea ID
        id: u32,
    },
    /// Start working on a task (creates branch, moves to Doing)
    Start {
        /// Task ID
        id: u32,
    },
    /// Mark a task as done (commits, pushes, moves to Done)
    Done {
        /// Task ID
        id: u32,
        /// Optional commit message
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Submit a task for review (push branch, create PR, move to Review)
    Submit {
        /// Task ID
        id: u32,
    },
    /// Check PR status and update task accordingly
    Review {
        /// Task ID
        id: u32,
    },
    /// Open interactive board view
    Board,
    /// Export tasks
    Export {
        /// Export format
        #[arg(short, long, value_enum)]
        format: ExportFormat,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum ExportFormat {
    Csv,
    Markdown,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init_command().await,
        Commands::Add { title, description } => add_command(title, description).await,
        Commands::List { column } => list_command(column).await,
        Commands::Move { id, column } => move_command(id, column).await,
        Commands::Comment { id, text } => comment_command(id, text).await,
        Commands::Idea { content } => idea_command(content).await,
        Commands::Promote { id } => promote_command(id).await,
        Commands::Start { id } => start_command(id).await,
        Commands::Done { id, message } => done_command(id, message).await,
        Commands::Submit { id } => submit_command(id).await,
        Commands::Review { id } => review_command(id).await,
        Commands::Board => board_command().await,
        Commands::Export { format } => export_command(format).await,
    }
}
