use db::{mysql::MySqlClient, postgres::PostgresClient, sqlite::SqliteClient, DbClient};
use errors::DbError;
use models::connections::{ConnectionConfig, DbType};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod db;
pub mod errors;
pub mod models;

#[derive(Default)]
pub struct DbManager {
    pub connections: Arc<Mutex<Vec<Box<dyn DbClient + Send + Sync>>>>,
}

impl DbManager {
    pub fn new() -> Self {
        DbManager {
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_connection(&self, config: ConnectionConfig) -> Result<(), DbError> {
        match config.db_type {
            DbType::Postgres => {
                let client = PostgresClient::connect(&config.database_url).await?;
                self.connections.lock().await.push(Box::new(client));
            }
            DbType::MySql => {
                let client = MySqlClient::connect(&config.database_url).await?;
                self.connections.lock().await.push(Box::new(client));
            }
            DbType::Sqlite => {
                let client = SqliteClient::connect(&config.database_url).await?;
                self.connections.lock().await.push(Box::new(client));
            }
        }

        Ok(())
    }
}
