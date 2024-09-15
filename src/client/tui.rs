use crate::db::postgres::PostgresClient;
use crate::models::connections::DbType;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io::{self, stdout};
use tokio::runtime::Runtime;

pub struct DatabaseClientUI {
    db_type: Option<DbType>,
    connection_string: String,
    connected: bool,
    client: Option<Box<dyn DatabaseClient>>,
    sql_input: String,
    sql_output: Option<String>,
}

#[async_trait::async_trait]
pub trait DatabaseClient {
    async fn list_tables(&self) -> Result<Vec<String>, io::Error>;
    async fn execute_query(&self, query: &str) -> Result<String, io::Error>;
}

#[async_trait::async_trait]
impl DatabaseClient for PostgresClient {
    async fn list_tables(&self) -> Result<Vec<String>, io::Error> {
        self.list_tables()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    }

    async fn execute_query(&self, query: &str) -> Result<String, io::Error> {
        self.execute_query(query)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    }
}

impl DatabaseClientUI {
    pub fn new() -> Self {
        Self {
            db_type: None,
            connection_string: String::new(),
            connected: false,
            client: None,
            sql_input: String::new(),
            sql_output: None,
        }
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        let stdout = stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| {
                let size = f.area(); // Исправлено на .area()
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(10),
                            Constraint::Percentage(10),
                            Constraint::Percentage(20),
                            Constraint::Percentage(50),
                            Constraint::Percentage(10),
                        ]
                        .as_ref(),
                    )
                    .split(size);

                let db_type_list = vec![
                    ListItem::new("1. PostgreSQL"),
                    ListItem::new("2. MySQL"),
                    ListItem::new("3. SQLite"),
                ];

                let db_type_widget = List::new(db_type_list)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Select DB Type"),
                    )
                    .style(Style::default().fg(Color::White));

                f.render_widget(db_type_widget, chunks[0]);

                let connection_string_widget = Paragraph::new(self.connection_string.as_str()) // Явное указание типа
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Connection String"),
                    );

                f.render_widget(connection_string_widget, chunks[1]);

                if self.connected {
                    let tables = match self.get_table_list() {
                        Ok(tables) => tables,
                        Err(_) => vec!["Failed to load tables".to_string()],
                    };

                    let table_items: Vec<ListItem> = tables
                        .iter()
                        .map(|t| ListItem::new(t.to_string()))
                        .collect();

                    let table_list = List::new(table_items)
                        .block(Block::default().borders(Borders::ALL).title("Tables"))
                        .style(Style::default().fg(Color::White));

                    f.render_widget(table_list, chunks[2]);
                }

                let sql_input_widget = Paragraph::new(self.sql_input.as_str()) // Явное указание типа
                    .block(Block::default().borders(Borders::ALL).title("SQL Query"));

                f.render_widget(sql_input_widget, chunks[3]);

                if let Some(output) = &self.sql_output {
                    let sql_output_widget = Paragraph::new(output.as_str()) // Явное указание типа
                        .block(Block::default().borders(Borders::ALL).title("Query Result"));
                    f.render_widget(sql_output_widget, chunks[4]);
                }
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('1') => self.db_type = Some(DbType::Postgres),
                        KeyCode::Char('2') => self.db_type = Some(DbType::MySql),
                        KeyCode::Char('3') => self.db_type = Some(DbType::Sqlite),
                        KeyCode::Enter => {
                            if !self.connected {
                                self.connect();
                            } else {
                                self.execute_query();
                            }
                        }
                        KeyCode::Char(c) => {
                            if !self.connected {
                                self.connection_string.push(c);
                            } else {
                                self.sql_input.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if !self.connected {
                                self.connection_string.pop();
                            } else {
                                self.sql_input.pop();
                            }
                        }
                        KeyCode::Char('q') => break,
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn connect(&mut self) {
        let rt = Runtime::new().unwrap();
        let db_type = self.db_type.clone();
        let connection_string = self.connection_string.clone();

        match db_type {
            Some(DbType::Postgres) => {
                let client = rt.block_on(async {
                    PostgresClient::connect(&connection_string)
                        .await
                        .map(Box::new)
                        .map(|c| c as Box<dyn DatabaseClient>) // Приведение к динамическому объекту
                        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to connect"))
                });

                self.client = client.ok();
                self.connected = self.client.is_some();
            }
            Some(DbType::MySql) => {
                // Добавить логику для MySQL
            }
            Some(DbType::Sqlite) => {
                // Добавить логику для SQLite
            }
            None => {}
        }
    }

    fn get_table_list(&self) -> Result<Vec<String>, io::Error> {
        if let Some(client) = &self.client {
            let rt = Runtime::new().unwrap();
            rt.block_on(async { client.list_tables().await })
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Not connected to database",
            ))
        }
    }

    fn execute_query(&mut self) {
        if let Some(client) = &self.client {
            let query = self.sql_input.clone();
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(async { client.execute_query(&query).await });
            self.sql_output = result.ok();
        }
    }
}
