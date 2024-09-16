use crate::DbManager;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use std::sync::Arc;

pub struct DatabaseClientUI {
    db_manager: Arc<DbManager>,
}

impl DatabaseClientUI {
    pub fn new(db_manager: Arc<DbManager>) -> Self {
        Self { db_manager }
    }

    pub async fn run(&self) -> Result<(), io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.ui_loop(&mut terminal).await;

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn ui_loop(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        let tables = self.fetch_tables().await.unwrap_or_else(|_| vec![]); // Получаем список таблиц

        loop {
            terminal.draw(|f| {
                let size = f.area();

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(20),
                            Constraint::Percentage(60),
                            Constraint::Percentage(20),
                        ]
                        .as_ref(),
                    )
                    .split(size);

                let block = Block::default()
                    .title("Database Manager")
                    .borders(Borders::ALL);
                f.render_widget(block, chunks[0]);

                let table_list: Vec<ListItem> = tables
                    .iter()
                    .map(|table| ListItem::new(table.clone()))
                    .collect();
                let tables_widget = List::new(table_list)
                    .block(Block::default().borders(Borders::ALL).title("Tables"))
                    .style(Style::default().fg(Color::White));

                f.render_widget(tables_widget, chunks[1]);

                let input = Paragraph::new("SQL Query Input")
                    .block(Block::default().borders(Borders::ALL).title("Query"));

                f.render_widget(input, chunks[2]);
            })?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }
    }

    async fn fetch_tables(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let connections = db_manager.connections.lock().await;
        if let Some(client) = connections.first() {
            let tables = client.list_tables().await?;
            Ok(tables)
        } else {
            Err("No database connection found".into())
        }
    }
}

