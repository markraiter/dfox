use std::collections::HashMap;

use dfox_lib::models::schema::TableSchema;

mod mysql;
mod postgres;

pub trait PostgresUI {
    async fn execute_sql_query(
        &mut self,
        query: &str,
    ) -> Result<(Vec<HashMap<String, serde_json::Value>>, Option<String>), Box<dyn std::error::Error>>;
    async fn describe_table(
        &self,
        table_name: &str,
    ) -> Result<TableSchema, Box<dyn std::error::Error>>;
    async fn fetch_databases(&self) -> Result<Vec<String>, Box<dyn std::error::Error>>;
    async fn fetch_tables(&self) -> Result<Vec<String>, Box<dyn std::error::Error>>;
    async fn update_tables(&mut self);
    async fn connect_to_selected_db(
        &mut self,
        db_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn connect_to_default_db(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait MySQLUI {
    async fn execute_sql_query(
        &mut self,
        query: &str,
    ) -> Result<(Vec<HashMap<String, serde_json::Value>>, Option<String>), Box<dyn std::error::Error>>;
    async fn describe_table(
        &self,
        table_name: &str,
    ) -> Result<TableSchema, Box<dyn std::error::Error>>;
    async fn fetch_databases(&self) -> Result<Vec<String>, Box<dyn std::error::Error>>;
    async fn fetch_tables(&self) -> Result<Vec<String>, Box<dyn std::error::Error>>;
    async fn update_tables(&mut self);
    async fn connect_to_selected_db(
        &mut self,
        db_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn connect_to_default_db(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
