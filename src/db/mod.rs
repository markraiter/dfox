use crate::{errors::DbError, models::schema::TableSchema};
use async_trait::async_trait;

pub mod postgres;

#[async_trait]
pub trait DbClient {
    async fn execute(&self, query: &str) -> Result<(), DbError>;
    async fn query(&self, query: &str) -> Result<Vec<serde_json::Value>, DbError>;
    async fn begin_transaction<'a>(&'a self) -> Result<Box<dyn Transaction + 'a>, DbError>;
    async fn list_tables(&self) -> Result<Vec<String>, DbError>;
    async fn describe_table(&self, table_name: &str) -> Result<TableSchema, DbError>;
}

#[async_trait]
pub trait Transaction {
    async fn execute(&mut self, query: &str) -> Result<(), DbError>;
    async fn commit(self: Box<Self>) -> Result<(), DbError>;
    async fn rollback(self: Box<Self>) -> Result<(), DbError>;
}
