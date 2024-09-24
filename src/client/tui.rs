use crate::db::{postgres::PostgresClient, DbClient};
use crate::DbManager;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Modifier;
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
    connection_string: String,
    current_screen: ScreenState,
    selected_db_type: usize,
}

enum ScreenState {
    DbTypeSelection,
    ConnectionInput,
    TableView,
}

impl DatabaseClientUI {
    pub fn new(db_manager: Arc<DbManager>) -> Self {
        Self {
            db_manager,
            connection_string: String::new(),
            current_screen: ScreenState::DbTypeSelection,
            selected_db_type: 0,
        }
    }

    pub async fn run(&mut self) -> Result<(), io::Error> {
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
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        loop {
            match self.current_screen {
                ScreenState::DbTypeSelection => self.db_type_selection_screen(terminal).await?,
                ScreenState::ConnectionInput => self.connection_input_screen(terminal).await?,
                ScreenState::TableView => self.table_view_screen(terminal).await?,
            }

            if let Event::Key(key) = event::read()? {
                match self.current_screen {
                    ScreenState::DbTypeSelection => match key.code {
                        KeyCode::Up => {
                            if self.selected_db_type > 0 {
                                self.selected_db_type -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if self.selected_db_type < 2 {
                                self.selected_db_type += 1;
                            }
                        }
                        KeyCode::Enter => {
                            self.current_screen = ScreenState::ConnectionInput;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    ScreenState::ConnectionInput => match key.code {
                        KeyCode::Char(c) => {
                            self.connection_string.push(c);
                        }
                        KeyCode::Backspace => {
                            self.connection_string.pop();
                        }
                        KeyCode::Enter => {
                            if !self.connection_string.is_empty() {
                                let result = self.connect_to_db().await;
                                if result.is_ok() {
                                    self.current_screen = ScreenState::TableView;
                                }
                            }
                        }
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    ScreenState::TableView => {
                        if key.code == KeyCode::Char('q') {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    async fn db_type_selection_screen(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        let db_types = ["Postgres", "MySQL", "SQLite"];
        let db_type_list: Vec<ListItem> = db_types
            .iter()
            .enumerate()
            .map(|(i, &db)| {
                if i == self.selected_db_type {
                    ListItem::new(db).style(
                        Style::default()
                            .bg(Color::Yellow)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    ListItem::new(db).style(Style::default().fg(Color::White))
                }
            })
            .collect();

        terminal.draw(|f| {
            let size = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(30),
                        Constraint::Percentage(40),
                        Constraint::Percentage(30),
                    ]
                    .as_ref(),
                )
                .split(size);

            let horizontal_layout = centered_rect(50, chunks[1]);

            let block = Block::default()
                .title("Select Database Type")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center);

            let db_type_widget = List::new(db_type_list).block(block).highlight_style(
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );

            f.render_widget(db_type_widget, horizontal_layout);
        })?;

        Ok(())
    }

    async fn connection_input_screen(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        terminal.draw(|f| {
            let size = f.area();
            let block = Block::default()
                .title("Enter DB Connection String")
                .borders(Borders::ALL);
            let connection_input = Paragraph::new(self.connection_string.as_str())
                .block(block)
                .style(Style::default().fg(Color::White));

            f.render_widget(connection_input, size);
        })?;
        Ok(())
    }

    async fn table_view_screen(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        let tables = self.fetch_tables().await.unwrap_or_else(|_| vec![]);

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

        Ok(())
    }

    async fn connect_to_db(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let mut connections = db_manager.connections.lock().await;
        let client = PostgresClient::connect(&self.connection_string).await?;
        connections.push(Box::new(client) as Box<dyn DbClient + Send + Sync>);
        Ok(())
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

fn centered_rect(width_percent: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - width_percent) / 2),
                Constraint::Percentage(width_percent),
                Constraint::Percentage((100 - width_percent) / 2),
            ]
            .as_ref(),
        )
        .split(area);

    popup_layout[1]
}
