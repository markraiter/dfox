use dfox_lib::{
    db::{postgres::PostgresClient, DbClient},
    models::schema::TableSchema,
};

use crate::ui::DatabaseClientUI;

impl DatabaseClientUI {
    pub async fn describe_table(
        &self,
        table_name: &str,
    ) -> Result<TableSchema, Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let connections = db_manager.connections.lock().await;
        if let Some(client) = connections.first() {
            let schema = client.describe_table(table_name).await?;
            Ok(schema)
        } else {
            Err("Some error occures".into())
        }
    }

    pub async fn fetch_databases(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let connections = db_manager.connections.lock().await;
        if let Some(client) = connections.first() {
            let databases = client.list_databases().await?;
            Ok(databases)
        } else {
            Err("No database connection found".into())
        }
    }

    pub async fn fetch_tables(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let db_manager = self.db_manager.clone();
        let connections = db_manager.connections.lock().await;

        if let Some(client) = connections.first() {
            let tables = client.list_tables().await?;
            return Ok(tables);
        }

        Ok(vec![])
    }

    pub async fn update_tables(&mut self) {
        match self.fetch_tables().await {
            Ok(tables) => {
                self.tables = tables;
                self.selected_table = 0; // Сброс индекса выбранной таблицы на 0
            }
            Err(err) => {
                println!("Error fetching tables: {}", err);
                self.tables = Vec::new(); // Очистить список таблиц при ошибке
                self.selected_table = 0; // Сброс индекса, если таблицы отсутствуют
            }
        }
    }

    pub async fn connect_to_selected_db(
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

    pub async fn connect_to_default_db(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
}
