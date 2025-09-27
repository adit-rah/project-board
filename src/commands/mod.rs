use anyhow::{Result, bail, Context};
use std::path::PathBuf;
use std::fs;

use crate::db::Database;
use crate::git::GitRepo;
use crate::github::{GitHubClient, extract_github_info};
use crate::ExportFormat;

pub async fn init_command() -> Result<()> {
    println!("🚀 Initializing ProjectBoard...");

    // Check if we're in a git repository
    let repo_path = std::env::current_dir()?;
    let git_repo = GitRepo::open(&repo_path)?;
    
    // Create .projectboard directory
    let pb_dir = repo_path.join(".projectboard");
    if pb_dir.exists() {
        bail!("ProjectBoard already initialized in this repository");
    }
    
    fs::create_dir_all(&pb_dir)
        .context("Failed to create .projectboard directory")?;

    // Create SQLite database
    let db_path = pb_dir.join("board.sqlite");
    let db = Database::new(&db_path).await?;
    
    // Run migrations
    db.migrate().await?;
    
    // Create default columns
    let columns = db.create_default_columns().await?;
    println!("📋 Created default columns:");
    for column in &columns {
        println!("  - {}", column.name);
    }
    
    // Create project entry
    let repo_name = repo_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();
    
    let project = db.create_project(&repo_name, repo_path.to_str().unwrap()).await?;
    
    // Log activity
    db.log_activity("project_initialized", Some(format!("Project: {}", project.name))).await?;
    
    println!("✅ ProjectBoard initialized successfully!");
    println!("   Database: {}", db_path.display());
    println!("   Use 'pb add \"Task title\"' to create your first task");
    
    Ok(())
}

pub async fn add_command(title: String, description: Option<String>) -> Result<()> {
    let db = get_database().await?;
    
    // Get the Backlog column
    let backlog_column = db.get_column_by_name("Backlog").await?
        .ok_or_else(|| anyhow::anyhow!("Backlog column not found"))?;
    
    // Create the task
    let task = db.create_task(&title, description.clone(), backlog_column.id).await?;
    
    // Log activity
    db.log_activity("task_created", Some(format!("Task #{}: {}", task.id, task.title))).await?;
    
    println!("📝 Created task #{}: {}", task.id, title);
    if let Some(desc) = description {
        println!("   Description: {}", desc);
    }
    println!("   Column: Backlog");
    
    Ok(())
}

pub async fn list_command(column_filter: Option<String>) -> Result<()> {
    let db = get_database().await?;
    
    let columns = db.get_columns().await?;
    
    if let Some(filter) = column_filter {
        // List tasks in specific column
        let column = db.get_column_by_name(&filter).await?
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", filter))?;
        
        let tasks = db.get_tasks(Some(column.id)).await?;
        
        println!("📋 {} ({} tasks)", column.name, tasks.len());
        for task in tasks {
            println!("  #{}: {}", task.id, task.title);
            if let Some(desc) = &task.description {
                println!("      {}", desc);
            }
            if let Some(branch) = &task.branch_name {
                println!("      🌿 Branch: {}", branch);
            }
            if let Some(pr) = &task.pr_url {
                println!("      🔗 PR: {}", pr);
            }
        }
    } else {
        // List all tasks grouped by column
        for column in columns {
            let tasks = db.get_tasks(Some(column.id)).await?;
            
            println!("\n📋 {} ({} tasks)", column.name, tasks.len());
            if tasks.is_empty() {
                println!("  (no tasks)");
            } else {
                for task in tasks {
                    println!("  #{}: {}", task.id, task.title);
                    if let Some(desc) = &task.description {
                        println!("      {}", desc);
                    }
                    if let Some(branch) = &task.branch_name {
                        println!("      🌿 Branch: {}", branch);
                    }
                    if let Some(pr) = &task.pr_url {
                        println!("      🔗 PR: {}", pr);
                    }
                }
            }
        }
    }
    
    Ok(())
}

pub async fn move_command(task_id: u32, column_name: String) -> Result<()> {
    let db = get_database().await?;
    
    // Get the task
    let task = db.get_task(task_id as i64).await?
        .ok_or_else(|| anyhow::anyhow!("Task #{} not found", task_id))?;
    
    // Get the target column
    let target_column = db.get_column_by_name(&column_name).await?
        .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", column_name))?;
    
    // Get current column for logging
    let current_column = db.get_columns().await?
        .into_iter()
        .find(|c| c.id == task.column_id)
        .ok_or_else(|| anyhow::anyhow!("Current column not found"))?;
    
    // Update the task
    db.update_task_column(task.id, target_column.id).await?;
    
    // Log activity
    db.log_activity(
        "task_moved", 
        Some(format!("Task #{}: {} → {}", task.id, current_column.name, target_column.name))
    ).await?;
    
    println!("📦 Moved task #{}: {} → {}", task_id, current_column.name, target_column.name);
    println!("   {}", task.title);
    
    Ok(())
}

pub async fn comment_command(task_id: u32, text: String) -> Result<()> {
    let db = get_database().await?;
    
    // Verify task exists
    let task = db.get_task(task_id as i64).await?
        .ok_or_else(|| anyhow::anyhow!("Task #{} not found", task_id))?;
    
    // Get current user (from git config)
    let author = get_git_user().unwrap_or_else(|| "unknown".to_string());
    
    // Create comment
    let comment = db.create_comment(task.id, &author, &text).await?;
    
    // Log activity
    db.log_activity(
        "comment_added", 
        Some(format!("Task #{}: comment by {}", task.id, author))
    ).await?;
    
    println!("💬 Added comment to task #{}: {}", task_id, task.title);
    println!("   {}: {}", author, text);
    
    Ok(())
}

pub async fn idea_command(content: String) -> Result<()> {
    let db = get_database().await?;
    
    let idea = db.create_idea(&content).await?;
    
    // Log activity
    db.log_activity("idea_created", Some(format!("Idea #{}: {}", idea.id, content))).await?;
    
    println!("💡 Created idea #{}: {}", idea.id, content);
    
    Ok(())
}

pub async fn promote_command(idea_id: u32) -> Result<()> {
    let db = get_database().await?;
    
    // Get the idea
    let idea = db.get_idea(idea_id as i64).await?
        .ok_or_else(|| anyhow::anyhow!("Idea #{} not found", idea_id))?;
    
    // Get Backlog column
    let backlog_column = db.get_column_by_name("Backlog").await?
        .ok_or_else(|| anyhow::anyhow!("Backlog column not found"))?;
    
    // Create task from idea
    let task = db.create_task(&idea.content, None, backlog_column.id).await?;
    
    // Delete the idea
    db.delete_idea(idea.id).await?;
    
    // Log activity
    db.log_activity(
        "idea_promoted", 
        Some(format!("Idea #{} → Task #{}: {}", idea_id, task.id, idea.content))
    ).await?;
    
    println!("🚀 Promoted idea #{} to task #{}: {}", idea_id, task.id, idea.content);
    
    Ok(())
}

pub async fn start_command(task_id: u32) -> Result<()> {
    let db = get_database().await?;
    let repo_path = std::env::current_dir()?;
    let git_repo = GitRepo::open(&repo_path)?;
    
    // Get the task
    let task = db.get_task(task_id as i64).await?
        .ok_or_else(|| anyhow::anyhow!("Task #{} not found", task_id))?;
    
    // Generate branch name
    let slug = task.title
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();
    let branch_name = format!("feature/{}-{}", task_id, slug);
    
    // Create and checkout branch
    git_repo.create_branch(&branch_name)?;
    git_repo.checkout_branch(&branch_name)?;
    
    // Update task with branch name
    db.update_task_branch(task.id, &branch_name).await?;
    
    // Move task to "Doing" column
    let doing_column = db.get_column_by_name("Doing").await?
        .ok_or_else(|| anyhow::anyhow!("Doing column not found"))?;
    db.update_task_column(task.id, doing_column.id).await?;
    
    // Log activity
    db.log_activity(
        "task_started", 
        Some(format!("Task #{}: created branch {}", task.id, branch_name))
    ).await?;
    
    println!("🚀 Started task #{}: {}", task_id, task.title);
    println!("   🌿 Created and checked out branch: {}", branch_name);
    println!("   📦 Moved to: Doing");
    
    Ok(())
}

pub async fn done_command(task_id: u32, message: Option<String>) -> Result<()> {
    let db = get_database().await?;
    let repo_path = std::env::current_dir()?;
    let git_repo = GitRepo::open(&repo_path)?;
    
    // Get the task
    let task = db.get_task(task_id as i64).await?
        .ok_or_else(|| anyhow::anyhow!("Task #{} not found", task_id))?;
    
    // Check if there are staged changes to commit
    if git_repo.has_staged_changes()? {
        let commit_message = message.unwrap_or_else(|| format!("Closes #{}: {}", task_id, task.title));
        git_repo.commit(&commit_message)?;
        println!("💾 Committed changes: {}", commit_message);
    }
    
    // Push branch if it exists
    if let Some(branch_name) = &task.branch_name {
        git_repo.push_branch(branch_name)?;
        println!("📤 Pushed branch: {}", branch_name);
    }
    
    // Move task to "Done" column
    let done_column = db.get_column_by_name("Done").await?
        .ok_or_else(|| anyhow::anyhow!("Done column not found"))?;
    db.update_task_column(task.id, done_column.id).await?;
    
    // Log activity
    db.log_activity(
        "task_completed", 
        Some(format!("Task #{}: {}", task.id, task.title))
    ).await?;
    
    println!("✅ Completed task #{}: {}", task_id, task.title);
    println!("   📦 Moved to: Done");
    
    Ok(())
}

pub async fn submit_command(task_id: u32) -> Result<()> {
    let db = get_database().await?;
    let repo_path = std::env::current_dir()?;
    let git_repo = GitRepo::open(&repo_path)?;
    
    // Get the task
    let task = db.get_task(task_id as i64).await?
        .ok_or_else(|| anyhow::anyhow!("Task #{} not found", task_id))?;
    
    let branch_name = task.branch_name
        .ok_or_else(|| anyhow::anyhow!("Task #{} has no associated branch", task_id))?;
    
    // Push branch
    git_repo.push_branch(&branch_name)?;
    println!("📤 Pushed branch: {}", branch_name);
    
    // Create GitHub PR
    let pr_url = if let Some(remote_url) = git_repo.get_remote_url()? {
        if let Some((owner, repo)) = extract_github_info(&remote_url) {
            let github = GitHubClient::new(owner, repo);
            let pr_title = format!("Task #{}: {}", task_id, task.title);
            let pr_body = task.description.unwrap_or_default();
            let base_branch = "main"; // TODO: get from config
            
            match github.create_pull_request(&pr_title, &pr_body, &branch_name, base_branch).await {
                Ok(url) => {
                    println!("🔗 Created PR: {}", url);
                    url
                }
                Err(e) => {
                    println!("⚠️  Failed to create PR: {}", e);
                    format!("https://github.com/{}/{}/compare/{}...{}", "owner", "repo", base_branch, branch_name)
                }
            }
        } else {
            println!("⚠️  Not a GitHub repository, cannot create PR");
            format!("Manual PR needed for branch: {}", branch_name)
        }
    } else {
        println!("⚠️  No remote URL found, cannot create PR");
        format!("Manual PR needed for branch: {}", branch_name)
    };
    
    // Update task with PR URL
    db.update_task_pr(task.id, &pr_url).await?;
    
    // Move task to "Review" column
    let review_column = db.get_column_by_name("Review").await?
        .ok_or_else(|| anyhow::anyhow!("Review column not found"))?;
    db.update_task_column(task.id, review_column.id).await?;
    
    // Log activity
    db.log_activity(
        "task_submitted", 
        Some(format!("Task #{}: PR created", task.id))
    ).await?;
    
    println!("📋 Submitted task #{} for review: {}", task_id, task.title);
    println!("   📦 Moved to: Review");
    
    Ok(())
}

pub async fn review_command(task_id: u32) -> Result<()> {
    let db = get_database().await?;
    
    // Get the task
    let task = db.get_task(task_id as i64).await?
        .ok_or_else(|| anyhow::anyhow!("Task #{} not found", task_id))?;
    
    if let Some(pr_url) = &task.pr_url {
        println!("🔍 Checking PR status for task #{}: {}", task_id, task.title);
        println!("   🔗 PR: {}", pr_url);
        println!("   ⏳ PR status check not implemented yet");
    } else {
        println!("❌ Task #{} has no associated PR", task_id);
    }
    
    Ok(())
}

pub async fn board_command() -> Result<()> {
    use crate::tui::run_board_interface;
    run_board_interface().await
}

pub async fn export_command(format: ExportFormat) -> Result<()> {
    let db = get_database().await?;
    let tasks = db.get_tasks(None).await?;
    let columns = db.get_columns().await?;
    
    match format {
        ExportFormat::Csv => {
            println!("ID,Title,Description,Column,Created,Updated,Branch,PR");
            for task in tasks {
                let column_name = columns.iter()
                    .find(|c| c.id == task.column_id)
                    .map(|c| &c.name)
                    .unwrap_or("Unknown");
                
                println!("{},{},{},{},{},{},{},{}",
                    task.id,
                    escape_csv(&task.title),
                    escape_csv(&task.description.unwrap_or_default()),
                    column_name,
                    task.created_at.format("%Y-%m-%d %H:%M:%S"),
                    task.updated_at.format("%Y-%m-%d %H:%M:%S"),
                    task.branch_name.unwrap_or_default(),
                    task.pr_url.unwrap_or_default()
                );
            }
        }
        ExportFormat::Markdown => {
            println!("# ProjectBoard Export\n");
            for column in columns {
                let column_tasks: Vec<_> = tasks.iter()
                    .filter(|t| t.column_id == column.id)
                    .collect();
                
                println!("## {} ({})\n", column.name, column_tasks.len());
                for task in column_tasks {
                    println!("- **#{}**: {}", task.id, task.title);
                    if let Some(desc) = &task.description {
                        println!("  - {}", desc);
                    }
                    if let Some(branch) = &task.branch_name {
                        println!("  - Branch: `{}`", branch);
                    }
                    if let Some(pr) = &task.pr_url {
                        println!("  - PR: {}", pr);
                    }
                    println!();
                }
            }
        }
    }
    
    Ok(())
}

// Helper functions
async fn get_database() -> Result<Database> {
    let repo_path = std::env::current_dir()?;
    let db_path = repo_path.join(".projectboard").join("board.sqlite");
    
    if !db_path.exists() {
        bail!("ProjectBoard not initialized. Run 'pb init' first.");
    }
    
    Database::new(&db_path).await
}

fn get_git_user() -> Option<String> {
    // Try to get git user name
    std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

fn escape_csv(text: &str) -> String {
    if text.contains(',') || text.contains('"') || text.contains('\n') {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_string()
    }
}
