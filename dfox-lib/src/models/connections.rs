use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DbType {
    Postgres,
    MySql,
    Sqlite,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectionConfig {
    pub db_type: DbType,
    pub database_url: String,
}
