use async_trait::async_trait;
use serde_json::Value;
use sqlx::{mysql::MySqlPoolOptions, Column, MySqlPool, Row};

use crate::{
    errors::DbError,
    models::schema::{ColumnSchema, TableSchema},
};

use super::{DbClient, Transaction};

pub struct MySqlClient {
    pub pool: MySqlPool,
}

impl MySqlClient {
    pub async fn connect(database_url: &str) -> Result<Self, DbError> {
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| DbError::Connection(e.to_string()))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl DbClient for MySqlClient {
    async fn execute(&self, query: &str) -> Result<(), DbError> {
        sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;
        Ok(())
    }

    async fn query(&self, query: &str) -> Result<Vec<serde_json::Value>, DbError> {
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;

        let results = rows
            .iter()
            .map(|row| {
                let json_map = row
                    .columns()
                    .iter()
                    .enumerate()
                    .map(|(i, column)| {
                        let column_name = column.name();
                        let value: Value = match row.try_get(i) {
                            Ok(val) => Value::String(val),
                            Err(_) => Value::Null,
                        };

                        (column_name.to_string(), value)
                    })
                    .collect();

                Value::Object(json_map)
            })
            .collect();

        Ok(results)
    }

    async fn begin_transaction<'a>(&'a self) -> Result<Box<dyn Transaction + 'a>, DbError> {
        let tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::Transaction(e.to_string()))?;
        Ok(Box::new(MySqlTransaction { tx }))
    }

    async fn list_databases(&self) -> Result<Vec<String>, DbError> {
        let query = "SHOW DATABASES";

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;

        let databases: Vec<String> = rows
            .iter()
            .map(|row| row.try_get::<String, _>("Database").unwrap_or_default())
            .collect();

        Ok(databases)
    }

    async fn list_tables(&self) -> Result<Vec<String>, DbError> {
        let query = "SHOW TABLES";

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;

        let tables: Vec<String> = rows
            .iter()
            .map(|row| row.try_get::<String, _>(0).unwrap_or_default())
            .collect();

        Ok(tables)
    }

    async fn describe_table(&self, table_name: &str) -> Result<TableSchema, DbError> {
        let query = format!("DESCRIBE {}", table_name);
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;

        let columns = rows
            .iter()
            .map(|row| ColumnSchema {
                name: row.try_get("Field").unwrap(),
                data_type: row.try_get("Type").unwrap(),
                is_nullable: row.try_get::<String, _>("Null").unwrap() == "YES",
                default: row.try_get("Default").ok(),
            })
            .collect();

        Ok(TableSchema {
            table_name: table_name.to_string(),
            columns,
            indexes: Vec::new(),
        })
    }
}

pub struct MySqlTransaction<'a> {
    tx: sqlx::Transaction<'a, sqlx::MySql>,
}

#[async_trait]
impl<'a> Transaction for MySqlTransaction<'a> {
    async fn execute_transaction(&mut self, query: &str) -> Result<(), DbError> {
        sqlx::query(query)
            .execute(&mut *self.tx)
            .await
            .map_err(|e| DbError::Transaction(e.to_string()))?;
        Ok(())
    }

    async fn commit_transaction(self: Box<Self>) -> Result<(), DbError> {
        self.tx
            .commit()
            .await
            .map_err(|e| DbError::Transaction(e.to_string()))
    }

    async fn rollback_transaction(self: Box<Self>) -> Result<(), DbError> {
        self.tx
            .rollback()
            .await
            .map_err(|e| DbError::Transaction(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mockall::{
        mock,
        predicate::{self, *},
    };

    mock! {
        pub DbClientMock {}

        #[async_trait]
        impl DbClient for DbClientMock {
            async fn execute(&self, query: &str) -> Result<(), DbError>;
            async fn query(&self, query: &str) -> Result<Vec<serde_json::Value>, DbError>;
            async fn list_databases(&self) -> Result<Vec<String>, DbError>;
            async fn list_tables(&self) -> Result<Vec<String>, DbError>;
            async fn describe_table(&self, table_name: &str) -> Result<TableSchema, DbError>;
            async fn begin_transaction<'a>(&'a self) -> Result<Box<dyn Transaction + 'a>, DbError>;
        }
    }

    #[tokio::test]
    async fn test_list_databases() {
        let mut mock_db = MockDbClientMock::new();

        mock_db
            .expect_list_databases()
            .returning(|| Ok(vec!["db1".to_string(), "db2".to_string()]));

        let databases = mock_db.list_databases().await.unwrap();
        assert_eq!(databases, vec!["db1".to_string(), "db2".to_string()]);
    }

    #[tokio::test]
    async fn test_list_tables() {
        let mut mock_db = MockDbClientMock::new();

        mock_db
            .expect_list_tables()
            .returning(|| Ok(vec!["users".to_string(), "orders".to_string()]));

        let tables = mock_db.list_tables().await.unwrap();
        assert_eq!(tables, vec!["users".to_string(), "orders".to_string()]);
    }

    #[tokio::test]
    async fn test_execute() {
        let mut mock_db = MockDbClientMock::new();

        mock_db
            .expect_execute()
            .with(predicate::eq(
                "INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com')",
            ))
            .returning(|_| Ok(()));

        let result = mock_db
            .execute("INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com')")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query() {
        let mut mock_db = MockDbClientMock::new();

        let row = serde_json::json!({
            "name": "Alice",
            "email": "alice@example.com"
        });
        mock_db
            .expect_query()
            .with(predicate::eq("SELECT name, email FROM users"))
            .returning(move |_| Ok(vec![row.clone()]));

        let result = mock_db
            .query("SELECT name, email FROM users")
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], "Alice");
    }

    #[tokio::test]
    async fn test_describe_table() {
        let mut mock_db = MockDbClientMock::new();

        let table_schema = TableSchema {
            table_name: "users".to_string(),
            columns: vec![
                ColumnSchema {
                    name: "id".to_string(),
                    data_type: "INT".to_string(),
                    is_nullable: false,
                    default: None,
                },
                ColumnSchema {
                    name: "name".to_string(),
                    data_type: "VARCHAR".to_string(),
                    is_nullable: true,
                    default: None,
                },
            ],
            indexes: Vec::new(),
        };

        mock_db
            .expect_describe_table()
            .with(predicate::eq("users"))
            .returning(move |_| Ok(table_schema.clone()));

        let result = mock_db.describe_table("users").await.unwrap();
        assert_eq!(result.table_name, "users");
        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.columns[0].name, "id");
        assert_eq!(result.columns[1].name, "name");
    }

    mock! {
        pub Transaction {}

        #[async_trait::async_trait]
        impl Transaction for Transaction {
            async fn execute_transaction(&mut self, query: &str) -> Result<(), DbError>;
            async fn commit_transaction(self: Box<Self>) -> Result<(), DbError>;
            async fn rollback_transaction(self: Box<Self>) -> Result<(), DbError>;
        }
    }

    #[tokio::test]
    async fn test_begin_transaction() {
        let mut mock_db = MockDbClientMock::new();
        let mut mock_tx = MockTransaction::new();

        mock_tx
            .expect_execute_transaction()
            .with(mockall::predicate::eq(
                "INSERT INTO users (name) VALUES ('Bob')",
            ))
            .returning(|_| Ok(()));

        let mock_tx = std::cell::RefCell::new(Some(mock_tx));

        mock_db
            .expect_begin_transaction()
            .returning(move || Ok(Box::new(mock_tx.borrow_mut().take().unwrap())));

        let mut transaction = mock_db.begin_transaction().await.unwrap();
        assert!(transaction
            .execute_transaction("INSERT INTO users (name) VALUES ('Bob')")
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        let mut mock_tx = MockTransaction::new();

        mock_tx.expect_commit_transaction().returning(|| Ok(()));

        let result = Box::new(mock_tx).commit_transaction().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        let mut mock_tx = MockTransaction::new();

        mock_tx.expect_rollback_transaction().returning(|| Ok(()));

        let result = Box::new(mock_tx).rollback_transaction().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_transaction() {
        let mut mock_tx = MockTransaction::new();

        mock_tx
            .expect_execute_transaction()
            .with(predicate::eq("INSERT INTO users (name) VALUES ('Alice')"))
            .returning(|_| Ok(()));

        let result = mock_tx
            .execute_transaction("INSERT INTO users (name) VALUES ('Alice')")
            .await;
        assert!(result.is_ok());
    }
}
