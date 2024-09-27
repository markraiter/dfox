use std::io;

use crossterm::event::KeyCode;

use super::{
    components::{InputField, ScreenState},
    DatabaseClientUI,
};

impl DatabaseClientUI {
    pub async fn handle_db_type_selection_input(&mut self, key: KeyCode) {
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
            KeyCode::Enter => self.current_screen = ScreenState::ConnectionInput,
            KeyCode::Char('q') => (),
            _ => {}
        }
    }

    pub async fn handle_input_event(&mut self, key: KeyCode) -> io::Result<()> {
        match key {
            KeyCode::Esc => {
                self.current_screen = ScreenState::DbTypeSelection;
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
                        let result = self.connect_to_default_db().await;
                        if result.is_ok() {
                            self.current_screen = ScreenState::DatabaseSelection;
                        }
                    }
                    _ => {}
                },
            },
        }
        Ok(())
    }

    pub async fn handle_database_selection_input(&mut self, key: KeyCode) -> io::Result<()> {
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
}
