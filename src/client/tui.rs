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
    connection_input: ConnectionInput,
    current_screen: ScreenState,
    selected_db_type: usize,
    databases: Vec<String>,
}

enum InputField {
    Username,
    Password,
    Hostname,
}

struct ConnectionInput {
    username: String,
    password: String,
    hostname: String,
    current_field: InputField,
}

impl ConnectionInput {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            hostname: String::new(),
            current_field: InputField::Username,
        }
    }
}

enum ScreenState {
    DbTypeSelection,
    DatabaseSelection,
    ConnectionInput,
    TableView,
}

impl DatabaseClientUI {
    pub fn new(db_manager: Arc<DbManager>) -> Self {
        Self {
            db_manager,
            connection_input: ConnectionInput::new(),
            current_screen: ScreenState::DbTypeSelection,
            selected_db_type: 0,
            databases: Vec::new(),
        }
    }

    pub async fn run_ui(&mut self) -> Result<(), io::Error> {
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
                ScreenState::DbTypeSelection => {
                    self.render_db_type_selection_screen(terminal).await?
                }
                ScreenState::ConnectionInput => {
                    self.render_connection_input_screen(terminal).await?
                }
                ScreenState::DatabaseSelection => {
                    self.render_database_selection_screen(terminal).await?
                }

                ScreenState::TableView => self.render_table_view_screen(terminal).await?,
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
                    ScreenState::ConnectionInput => {
                        self.handle_input_event(key.code).await?;
                    }
                    ScreenState::TableView => {
                        if key.code == KeyCode::Char('q') {
                            return Ok(());
                        }
                    }
                    ScreenState::DatabaseSelection => {
                        self.handle_database_selection_input(key.code).await?;
                    }
                }
            }
        }
    }

    async fn render_db_type_selection_screen(
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

    async fn render_connection_input_screen(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        terminal.draw(|f| {
            let size = f.area();
            let vertical_chunks = Layout::default()
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

            let horizontal_layout = centered_rect(50, vertical_chunks[1]);

            let block = Block::default()
                .title("Enter Connection Details")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center);

            let mut content = [
                format!("Username: {}", self.connection_input.username),
                format!(
                    "Password: {}",
                    "*".repeat(self.connection_input.password.len())
                ),
                format!("Hostname: {}", self.connection_input.hostname),
            ];

            content[self.current_input_index()].push_str(" <");

            let input_paragraph = Paragraph::new(content.join("\n"))
                .block(block)
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Left);

            f.render_widget(input_paragraph, horizontal_layout);
        })?;

        Ok(())
    }

    fn current_input_index(&self) -> usize {
        match self.connection_input.current_field {
            InputField::Username => 0,
            InputField::Password => 1,
            InputField::Hostname => 2,
        }
    }

    async fn handle_input_event(&mut self, key: KeyCode) -> io::Result<()> {
        match self.connection_input.current_field {
            InputField::Username => match key {
                KeyCode::Char(c) => self.connection_input.username.push(c),
                KeyCode::Backspace => {
                    self.connection_input.username.pop();
                }
                KeyCode::Enter => {
                    self.connection_input.current_field = InputField::Password;
                }
                _ => {}
            },
            InputField::Password => match key {
                KeyCode::Char(c) => self.connection_input.password.push(c),
                KeyCode::Backspace => {
                    self.connection_input.password.pop();
                }
                KeyCode::Enter => {
                    self.connection_input.current_field = InputField::Hostname;
                }
                _ => {}
            },
            InputField::Hostname => match key {
                KeyCode::Char(c) => self.connection_input.hostname.push(c),
                KeyCode::Backspace => {
                    self.connection_input.hostname.pop();
                }
                KeyCode::Enter => {
                    let result = self.connect_to_default_db().await;
                    if result.is_ok() {
                        self.current_screen = ScreenState::DatabaseSelection;
                    }
                }
                _ => {}
            },
        }
        Ok(())
    }

    async fn connect_to_selected_db(
        &mut self,
        db_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let mut connections = db_manager.connections.lock().await;
        connections.clear();

        let connection_string = format!(
            "postgres://{}:{}@{}/{}",
            self.connection_input.username,
            self.connection_input.password,
            self.connection_input.hostname,
            db_name,
        );

        let client = PostgresClient::connect(&connection_string).await?;
        connections.push(Box::new(client) as Box<dyn DbClient + Send + Sync>);

        Ok(())
    }

    async fn connect_to_default_db(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let mut connections = db_manager.connections.lock().await;

        let connection_string = format!(
            "postgres://{}:{}@{}/postgres",
            self.connection_input.username,
            self.connection_input.password,
            self.connection_input.hostname
        );

        let client = PostgresClient::connect(&connection_string).await?;
        connections.push(Box::new(client) as Box<dyn DbClient + Send + Sync>);

        Ok(())
    }

    async fn fetch_tables(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let connections = db_manager.connections.lock().await;

        if let Some(client) = connections.first() {
            let tables = client.list_tables().await?;
            return Ok(tables);
        }

        Ok(vec![])
    }

    async fn fetch_databases(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let connections = db_manager.connections.lock().await;
        if let Some(client) = connections.first() {
            let databases = client.list_databases().await?;
            Ok(databases)
        } else {
            Err("No database connection found".into())
        }
    }

    async fn handle_database_selection_input(&mut self, key: KeyCode) -> io::Result<()> {
        match key {
            KeyCode::Up => {
                if self.selected_db_type > 0 {
                    self.selected_db_type -= 1;
                }
            }
            KeyCode::Down => {
                if !self.databases.is_empty() && self.selected_db_type < self.databases.len() - 1 {
                    self.selected_db_type += 1;
                }
            }
            KeyCode::Enter => {
                let cloned = self.databases.clone();
                if let Some(db_name) = cloned.get(self.selected_db_type) {
                    if let Err(err) = self.connect_to_selected_db(db_name).await {
                        eprintln!("Error connecting to database: {}", err);
                    } else {
                        self.current_screen = ScreenState::TableView;
                    }
                }
            }
            KeyCode::Char('q') => {
                return Ok(());
            }
            _ => {}
        }
        Ok(())
    }

    async fn render_database_selection_screen(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        match self.fetch_databases().await {
            Ok(databases) => {
                self.databases = databases;
            }
            Err(_) => {
                self.databases = vec!["Error fetching databases".to_string()];
            }
        }

        let db_list: Vec<ListItem> = self
            .databases
            .iter()
            .enumerate()
            .map(|(i, db)| {
                if i == self.selected_db_type {
                    ListItem::new(db.clone()).style(
                        Style::default()
                            .bg(Color::Yellow)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    ListItem::new(db.clone()).style(Style::default().fg(Color::White))
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
                .title("Select Database")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center);

            let db_list_widget = List::new(db_list).block(block).highlight_style(
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );

            f.render_widget(db_list_widget, horizontal_layout);
        })?;

        Ok(())
    }

    async fn render_table_view_screen(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        let tables = self.fetch_tables().await.unwrap_or_else(|_| vec![]);

        terminal.draw(|f| {
            let size = f.area();

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(size);

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(chunks[1]);

            let tables_block = Block::default().borders(Borders::ALL).title("Tables");

            let table_list: Vec<ListItem> = tables
                .iter()
                .map(|table| ListItem::new(table.to_string()))
                .collect();

            let tables_widget = List::new(table_list).block(tables_block);

            let sql_query_block = Block::default().borders(Borders::ALL).title("SQL Query");

            let sql_query_widget = Paragraph::new("SELECT * FROM ...")
                .block(sql_query_block)
                .style(Style::default().fg(Color::White));

            let sql_result_block = Block::default().borders(Borders::ALL).title("Query Result");

            let sql_result_widget = Paragraph::new("Results will be shown here...")
                .block(sql_result_block)
                .style(Style::default().fg(Color::White));

            f.render_widget(tables_widget, chunks[0]);
            f.render_widget(sql_query_widget, right_chunks[0]);
            f.render_widget(sql_result_widget, right_chunks[1]);
        })?;

        Ok(())
    }
}

fn centered_rect(percent_x: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    popup_layout[1]
}
