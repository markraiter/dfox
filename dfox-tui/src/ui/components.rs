use std::{collections::HashMap, sync::Arc};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dfox_lib::{models::schema::TableSchema, DbManager};
use ratatui::{backend::CrosstermBackend, Terminal};
use serde_json::Value;
use std::io;

use super::screens::{
    render_connection_input_screen, render_database_selection_screen,
    render_db_type_selection_screen, render_table_view_screen,
};

pub struct DatabaseClientUI {
    pub db_manager: Arc<DbManager>,
    pub connection_input: ConnectionInput,
    pub current_screen: ScreenState,
    pub selected_db_type: usize,
    pub databases: Vec<String>,
    pub current_focus: FocusedWidget,
    pub selected_table: usize,
    pub tables: Vec<String>,
    pub sql_editor_content: String,
    pub sql_query_result: Vec<HashMap<String, Value>>,
    pub expanded_table: Option<usize>,
    pub table_schemas: HashMap<String, TableSchema>,
}

pub enum InputField {
    Username,
    Password,
    Hostname,
}

pub struct ConnectionInput {
    pub username: String,
    pub password: String,
    pub hostname: String,
    pub current_field: InputField,
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

pub enum ScreenState {
    DbTypeSelection,
    DatabaseSelection,
    ConnectionInput,
    TableView,
}

#[derive(Clone, PartialEq)]
pub enum FocusedWidget {
    TablesList,
    SqlEditor,
    QueryResult,
}

impl DatabaseClientUI {
    pub fn new(db_manager: Arc<DbManager>) -> Self {
        Self {
            db_manager,
            connection_input: ConnectionInput::new(),
            current_screen: ScreenState::DbTypeSelection,
            selected_db_type: 0,
            databases: Vec::new(),
            current_focus: FocusedWidget::TablesList,
            selected_table: 0,
            tables: Vec::new(),
            sql_editor_content: String::new(),
            sql_query_result: Vec::new(),
            expanded_table: None,
            table_schemas: HashMap::new(),
        }
    }

    pub fn current_input_index(&self) -> usize {
        match self.connection_input.current_field {
            InputField::Username => 0,
            InputField::Password => 1,
            InputField::Hostname => 2,
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
                    render_db_type_selection_screen(self, terminal).await?
                }
                ScreenState::ConnectionInput => {
                    render_connection_input_screen(self, terminal).await?
                }
                ScreenState::DatabaseSelection => {
                    render_database_selection_screen(self, terminal).await?
                }
                ScreenState::TableView => render_table_view_screen(self, terminal).await?,
            }

            if let Event::Key(key) = event::read()? {
                match self.current_screen {
                    ScreenState::DbTypeSelection => {
                        self.handle_db_type_selection_input(key.code).await;
                    }
                    ScreenState::ConnectionInput => {
                        self.handle_input_event(key.code).await?;
                    }
                    ScreenState::DatabaseSelection => {
                        self.handle_database_selection_input(key.code).await?;
                    }
                    ScreenState::TableView => {
                        if key.code == KeyCode::Esc {
                            return Ok(());
                        }

                        if let FocusedWidget::SqlEditor = self.current_focus {
                            self.handle_sql_editor_input(key.code, terminal).await;
                        } else {
                            self.handle_table_view_input(key.code, terminal).await;
                        }
                    }
                }
            }
        }
    }
}
