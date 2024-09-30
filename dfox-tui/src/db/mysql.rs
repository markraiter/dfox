use std::collections::HashMap;

use dfox_lib::db::{mysql::MySqlClient, DbClient};

use crate::ui::DatabaseClientUI;

use super::MySQLUI;

impl MySQLUI for DatabaseClientUI {
    async fn execute_sql_query(
        &mut self,
        query: &str,
    ) -> Result<Vec<std::collections::HashMap<String, serde_json::Value>>, Box<dyn std::error::Error>>
    {
        let db_manager = self.db_manager.clone();
        let connections = db_manager.connections.lock().await;

        if let Some(client) = connections.first() {
            let query_trimmed = query.trim();
            let query_upper = query_trimmed.to_uppercase();

            if query_upper.starts_with("SELECT") {
                let rows: Vec<serde_json::Value> = client.query(query_trimmed).await?;

                let hash_map_results: Vec<HashMap<String, serde_json::Value>> = rows
                    .into_iter()
                    .filter_map(|row| {
                        if let serde_json::Value::Object(map) = row {
                            Some(
                                map.into_iter()
                                    .collect::<HashMap<String, serde_json::Value>>(),
                            )
                        } else {
                            None
                        }
                    })
                    .collect();

                self.sql_query_result = hash_map_results.clone();
                Ok(hash_map_results)
            } else {
                client.execute(query_trimmed).await?;
                println!("Non-SELECT query executed successfully.");
                Ok(Vec::new())
            }
        } else {
            Err("No database connection available.".into())
        }
    }

    async fn describe_table(
        &self,
        table_name: &str,
    ) -> Result<dfox_lib::models::schema::TableSchema, Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let connections = db_manager.connections.lock().await;

        if let Some(client) = connections.first() {
            let schema = client.describe_table(table_name).await?;
            Ok(schema)
        } else {
            Err("No database connection available.".into())
        }
    }

    async fn fetch_databases(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let connections = db_manager.connections.lock().await;

        if let Some(client) = connections.first() {
            let databases = client.list_databases().await?;
            println!("Fetched databases: {:?}", databases);
            Ok(databases)
        } else {
            Err("No database connection available.".into())
        }
    }

    async fn fetch_tables(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let connections = db_manager.connections.lock().await;

        if let Some(client) = connections.first() {
            let tables = client.list_tables().await?;
            Ok(tables)
        } else {
            Err("No database connection available.".into())
        }
    }

    async fn update_tables(&mut self) {
        match self.fetch_tables().await {
            Ok(tables) => {
                self.tables = tables;
                self.selected_table = 0;
            }
            Err(err) => {
                println!("Error fetching tables: {}", err);
                self.tables = Vec::new();
                self.selected_table = 0;
            }
        }
    }

    async fn connect_to_selected_db(
        &mut self,
        db_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let mut connections = db_manager.connections.lock().await;
        connections.clear();

        let connection_string = format!(
            "mysql://{}:{}@{}/{}",
            self.connection_input.username,
            self.connection_input.password,
            self.connection_input.hostname,
            db_name,
        );

        let client = MySqlClient::connect(&connection_string).await?;
        connections.push(Box::new(client) as Box<dyn DbClient + Send + Sync>);

        Ok(())
    }

    async fn connect_to_default_db(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let mut connections = db_manager.connections.lock().await;

        let connection_string = format!(
            "mysql://{}:{}@{}/mysql",
            self.connection_input.username,
            self.connection_input.password,
            self.connection_input.hostname
        );

        let client = MySqlClient::connect(&connection_string).await?;
        connections.push(Box::new(client) as Box<dyn DbClient + Send + Sync>);

        Ok(())
    }
}
