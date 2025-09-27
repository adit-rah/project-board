use anyhow::Result;
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::db::Database;

pub async fn run_board_interface() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let db = get_database().await?;
    let app = App::new(db).await?;
    
    // Run the app
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

struct App {
    db: Database,
    columns: Vec<crate::db::Column>,
    tasks_by_column: std::collections::HashMap<i64, Vec<crate::db::Task>>,
    selected_column: usize,
}

impl App {
    async fn new(db: Database) -> Result<Self> {
        let columns = db.get_columns().await?;
        let mut tasks_by_column = std::collections::HashMap::new();
        
        for column in &columns {
            let tasks = db.get_tasks(Some(column.id)).await?;
            tasks_by_column.insert(column.id, tasks);
        }
        
        Ok(App {
            db,
            columns,
            tasks_by_column,
            selected_column: 0,
        })
    }
    
    fn next_column(&mut self) {
        if self.selected_column < self.columns.len() - 1 {
            self.selected_column += 1;
        }
    }
    
    fn previous_column(&mut self) {
        if self.selected_column > 0 {
            self.selected_column -= 1;
        }
    }
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Left => app.previous_column(),
                KeyCode::Right => app.next_column(),
                KeyCode::Char('r') => {
                    // Refresh data
                    app = App::new(app.db).await?;
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();
    
    // Create layout with header and main content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(size);
    
    // Header
    let header = Paragraph::new("ProjectBoard - Use ← → to navigate, 'r' to refresh, 'q' to quit")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(header, chunks[0]);
    
    // Main board
    let board_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            app.columns
                .iter()
                .map(|_| Constraint::Percentage((100 / app.columns.len()) as u16))
                .collect::<Vec<_>>(),
        )
        .split(chunks[1]);
    
    for (i, column) in app.columns.iter().enumerate() {
        let tasks = app.tasks_by_column.get(&column.id).unwrap_or(&vec![]);
        
        let items: Vec<ListItem> = tasks
            .iter()
            .map(|task| {
                let content = vec![Line::from(vec![
                    Span::styled(
                        format!("#{} ", task.id),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(&task.title),
                ])];
                ListItem::new(content)
            })
            .collect();
        
        let style = if i == app.selected_column {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{} ({})", column.name, tasks.len()))
                    .border_style(style),
            )
            .style(Style::default().fg(Color::White));
        
        f.render_widget(list, board_layout[i]);
    }
}

async fn get_database() -> Result<Database> {
    let repo_path = std::env::current_dir()?;
    let db_path = repo_path.join(".projectboard").join("board.sqlite");
    
    if !db_path.exists() {
        anyhow::bail!("ProjectBoard not initialized. Run 'pb init' first.");
    }
    
    Database::new(&db_path).await
}
