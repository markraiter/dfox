use std::{
    io::{self, stdout},
    process,
};

use crossterm::{
    event::{KeyCode, KeyModifiers},
    execute, terminal,
};
use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::db::{MySQLUI, PostgresUI};

use super::{
    components::{FocusedWidget, InputField, ScreenState},
    DatabaseClientUI, UIHandler, UIRenderer,
};

impl UIHandler for DatabaseClientUI {
    async fn handle_message_popup_input(&mut self) {
        self.current_screen = ScreenState::DbTypeSelection
    }

    async fn handle_db_type_selection_input(&mut self, key: KeyCode) {
        match key {
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
                if self.selected_db_type == 2 {
                    self.current_screen = ScreenState::MessagePopup;
                } else {
                    self.current_screen = ScreenState::ConnectionInput;
                }
            }
            KeyCode::Char('q') => {
                terminal::disable_raw_mode().unwrap();
                execute!(stdout(), terminal::LeaveAlternateScreen).unwrap();
                process::exit(0);
            }
            _ => {}
        }
    }

    async fn handle_input_event(&mut self, key: KeyCode) -> io::Result<()> {
        if let Some(_error_message) = &self.connection_error_message {
            match key {
                KeyCode::Enter | KeyCode::Esc => {
                    self.connection_error_message = None;
                }
                _ => {}
            }
        } else {
            match key {
                KeyCode::Esc => {
                    self.current_screen = ScreenState::DbTypeSelection;
                }
                KeyCode::Up => {
                    self.connection_input.current_field = match self.connection_input.current_field
                    {
                        InputField::Port => InputField::Hostname,
                        InputField::Hostname => InputField::Password,
                        InputField::Password => InputField::Username,
                        InputField::Username => InputField::Username,
                    };
                }
                KeyCode::Down => {
                    self.connection_input.current_field = match self.connection_input.current_field
                    {
                        InputField::Username => InputField::Password,
                        InputField::Password => InputField::Hostname,
                        InputField::Hostname => InputField::Port,
                        InputField::Port => InputField::Port,
                    };
                }
                _ => match self.connection_input.current_field {
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
                            self.connection_input.current_field = InputField::Port;
                        }
                        _ => {}
                    },
                    InputField::Port => match key {
                        KeyCode::Char(c) => self.connection_input.port.push(c),
                        KeyCode::Backspace => {
                            self.connection_input.port.pop();
                        }
                        KeyCode::Enter => match self.selected_db_type {
                            0 => {
                                let result = PostgresUI::connect_to_default_db(self).await;
                                if result.is_ok() {
                                    self.current_screen = ScreenState::DatabaseSelection;
                                }
                            }
                            1 => {
                                let result = MySQLUI::connect_to_default_db(self).await;
                                if result.is_ok() {
                                    self.current_screen = ScreenState::DatabaseSelection;
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                },
            }
        }
        Ok(())
    }

    async fn handle_database_selection_input(&mut self, key: KeyCode) -> io::Result<()> {
        match key {
            KeyCode::Up => {
                if self.selected_database > 0 {
                    self.selected_database -= 1;
                }
            }
            KeyCode::Down => {
                if !self.databases.is_empty() && self.selected_database < self.databases.len() - 1 {
                    self.selected_database += 1;
                }
            }
            KeyCode::Enter => {
                let cloned = self.databases.clone();
                if let Some(db_name) = cloned.get(self.selected_database) {
                    match self.selected_db_type {
                        0 => {
                            if let Err(err) =
                                PostgresUI::connect_to_selected_db(self, db_name).await
                            {
                                eprintln!("Error connecting to PostgreSQL database: {}", err);
                            } else {
                                self.current_screen = ScreenState::TableView;
                            }
                        }
                        1 => {
                            if let Err(err) = MySQLUI::connect_to_selected_db(self, db_name).await {
                                eprintln!("Error connecting to MySQL database: {}", err);
                            } else {
                                self.current_screen = ScreenState::TableView;
                            }
                        }
                        _ => {
                            eprintln!("Unsupported database type");
                        }
                    }
                }
            }
            KeyCode::Char('q') => {
                terminal::disable_raw_mode().unwrap();
                execute!(stdout(), terminal::LeaveAlternateScreen).unwrap();
                process::exit(0);
            }
            _ => {}
        }
        match self.selected_db_type {
            0 => PostgresUI::update_tables(self).await,
            1 => MySQLUI::update_tables(self).await,
            _ => (),
        }

        Ok(())
    }

    async fn handle_table_view_input(
        &mut self,
        key: KeyCode,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) {
        match key {
            KeyCode::F(1) => {
                self.current_screen = ScreenState::DatabaseSelection;
                self.sql_editor_content.clear();
                self.sql_query_result.clear();
                if let Err(err) = UIRenderer::render_database_selection_screen(self, terminal).await
                {
                    eprintln!("Error rendering database selection screen: {}", err);
                }
            }
            KeyCode::Tab => self.cycle_focus(),
            KeyCode::Up => {
                if let FocusedWidget::TablesList = self.current_focus {
                    self.move_selection_up();
                }
            }
            KeyCode::Down => {
                if let FocusedWidget::TablesList = self.current_focus {
                    self.move_selection_down();
                }
            }
            KeyCode::Enter => {
                if let FocusedWidget::TablesList = self.current_focus {
                    if self.tables.is_empty() {
                        println!("No tables available.");
                        return;
                    }

                    if self.selected_table < self.tables.len() {
                        let selected_table = self.tables[self.selected_table].clone();

                        if Some(self.selected_table) == self.expanded_table {
                            self.expanded_table = None;
                        } else {
                            match self.selected_db_type {
                                0 => {
                                    match PostgresUI::describe_table(self, &selected_table).await {
                                        Ok(table_schema) => {
                                            self.table_schemas.insert(
                                                selected_table.clone(),
                                                table_schema.clone(),
                                            );
                                            self.expanded_table = Some(self.selected_table);

                                            if let Err(err) = UIRenderer::render_table_schema(
                                                self,
                                                terminal,
                                                &table_schema,
                                            )
                                            .await
                                            {
                                                eprintln!("Error rendering table schema: {}", err);
                                            }
                                        }
                                        Err(err) => {
                                            eprintln!("Error describing table: {}", err);
                                        }
                                    }
                                }
                                1 => match MySQLUI::describe_table(self, &selected_table).await {
                                    Ok(table_schema) => {
                                        self.table_schemas
                                            .insert(selected_table.clone(), table_schema.clone());
                                        self.expanded_table = Some(self.selected_table);

                                        if let Err(err) = UIRenderer::render_table_schema(
                                            self,
                                            terminal,
                                            &table_schema,
                                        )
                                        .await
                                        {
                                            eprintln!("Error rendering table schema: {}", err);
                                        }
                                    }
                                    Err(err) => {
                                        eprintln!("Error describing table: {}", err);
                                    }
                                },
                                _ => (),
                            }
                        }
                    } else {
                        eprintln!("Selected table index out of bounds.");
                    }
                }
            }
            _ => {}
        }
    }

    async fn handle_sql_editor_input(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) {
        match (key, modifiers) {
            (KeyCode::Tab, _) => self.cycle_focus(),
            (KeyCode::F(5), _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                if !self.sql_editor_content.is_empty() {
                    self.sql_query_error = None;
                    let sql_content = self.sql_editor_content.clone();
                    match self.selected_db_type {
                        0 => match PostgresUI::execute_sql_query(self, &sql_content).await {
                            Ok((result, success_message)) => {
                                self.sql_query_result = result;
                                self.sql_query_success_message = success_message;
                                self.sql_query_error = None;
                            }
                            Err(err) => {
                                self.sql_query_error = Some(err.to_string());
                                self.sql_query_result.clear();
                            }
                        },
                        1 => match MySQLUI::execute_sql_query(self, &sql_content).await {
                            Ok((result, success_message)) => {
                                self.sql_query_result = result;
                                self.sql_query_success_message = success_message;
                                self.sql_query_error = None;
                            }
                            Err(err) => {
                                self.sql_query_error = Some(err.to_string());
                                self.sql_query_result.clear();
                            }
                        },
                        _ => (),
                    }
                    self.sql_editor_content.clear();
                }

                PostgresUI::update_tables(self).await;
            }
            (KeyCode::Enter, _) => {
                self.sql_editor_content.push('\n');
            }
            (KeyCode::Char(c), _) => {
                self.sql_editor_content.push(c);
            }
            (KeyCode::Backspace, _) => {
                self.sql_editor_content.pop();
            }
            (KeyCode::F(1), _) => {
                self.current_screen = ScreenState::DatabaseSelection;
                self.sql_editor_content.clear();
                self.sql_query_result.clear();
                if let Err(err) = UIRenderer::render_database_selection_screen(self, terminal).await
                {
                    eprintln!("Error rendering database selection screen: {}", err);
                }
                return;
            }
            _ => {}
        }
        if let Err(err) = UIRenderer::render_table_view_screen(self, terminal).await {
            eprintln!("Error rendering UI: {}", err);
        }
    }
}

impl DatabaseClientUI {
    pub fn cycle_focus(&mut self) {
        self.current_focus = match self.current_focus {
            FocusedWidget::TablesList => FocusedWidget::SqlEditor,
            FocusedWidget::SqlEditor => FocusedWidget::TablesList,
            FocusedWidget::_QueryResult => FocusedWidget::TablesList,
        };
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_table > 0 {
            self.selected_table -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.selected_table < self.databases.len().saturating_sub(1) {
            self.selected_table += 1;
        }
    }
}
