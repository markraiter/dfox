use std::fs::File;

use async_trait::async_trait;
use csv::{Reader, Writer};
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, Column, PgPool, Row};

use crate::{
    errors::DbError,
    models::schema::{ColumnSchema, TableSchema},
};

use super::{DbClient, Transaction};

pub struct PostgresClient {
    pub pool: PgPool,
}

impl PostgresClient {
    pub async fn connect(database_url: &str) -> Result<Self, DbError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| DbError::Connection(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Data import from CSV into table.
    pub async fn import_csv(&self, table: &str, file_path: &str) -> Result<(), DbError> {
        let file = File::open(file_path).map_err(|e| DbError::Import(e.to_string()))?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.records() {
            let record = result.map_err(|e| DbError::Import(e.to_string()))?;

            let values: Vec<String> = record
                .iter()
                .map(|val| {
                    if val.parse::<i64>().is_ok() {
                        val.to_string()
                    } else {
                        format!("'{}'", val)
                    }
                })
                .collect();
            let values_str = values.join(", ");

            let query_str = format!("INSERT INTO {} VALUES ({})", table, values_str);
            sqlx::query(&query_str)
                .execute(&self.pool)
                .await
                .map_err(DbError::Sqlx)?;
        }

        Ok(())
    }

    /// Data export from table to CSV.
    pub async fn export_to_csv(&self, table: &str, file_path: &str) -> Result<(), DbError> {
        let rows = sqlx::query(&format!("SELECT * FROM {}", table))
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;

        let file = File::create(file_path).map_err(|e| DbError::Export(e.to_string()))?;
        let mut wtr = Writer::from_writer(file);

        for row in rows {
            let mut csv_row = Vec::new();

            for column in row.columns() {
                let value: String = row
                    .try_get(column.name())
                    .map(|val: Option<String>| val.unwrap_or_else(|| "NULL".to_string()))
                    .unwrap_or("NULL".to_string());
                csv_row.push(value);
            }

            wtr.write_record(&csv_row)
                .map_err(|e| DbError::Export(e.to_string()))?;
        }

        wtr.flush().map_err(|e| DbError::Export(e.to_string()))?;

        Ok(())
    }

    pub async fn create_table(
        &self,
        table_name: &str,
        columns: &[ColumnSchema],
    ) -> Result<(), DbError> {
        let mut query = format!("CREATE TABLE {} (", table_name);

        for (i, column) in columns.iter().enumerate() {
            query.push_str(&format!(
                "{} {} {}{}",
                column.name,
                column.data_type,
                if column.is_nullable { "" } else { "NOT NULL" },
                if let Some(default) = &column.default {
                    format!(" DEFAULT {}", default)
                } else {
                    "".to_string()
                }
            ));
            if i < columns.len() - 1 {
                query.push_str(", ");
            }
        }
        query.push_str(");");

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;

        Ok(())
    }

    pub async fn drop_table(&self, table_name: &str) -> Result<(), DbError> {
        let query = format!("DROP TABLE IF EXISTS {}", table_name);
        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;

        Ok(())
    }

    pub async fn create_index(&self, table_name: &str, column_name: &str) -> Result<(), DbError> {
        let query = format!(
            "CREATE INDEX idx_{}_{} ON {} ({})",
            table_name, column_name, table_name, column_name
        );
        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;
        Ok(())
    }

    pub async fn drop_index(&self, index_name: &str) -> Result<(), DbError> {
        let query = format!("DROP INDEX IF EXISTS {}", index_name);
        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;
        Ok(())
    }

    pub async fn add_unique_constraint(
        &self,
        table_name: &str,
        column_name: &str,
    ) -> Result<(), DbError> {
        let query = format!(
            "ALTER TABLE {} ADD CONSTRAINT unique_{}_{} UNIQUE ({})",
            table_name, table_name, column_name, column_name
        );
        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;
        Ok(())
    }

    pub async fn add_foreign_key(
        &self,
        table_name: &str,
        column_name: &str,
        foreign_table: &str,
        foreign_column: &str,
    ) -> Result<(), DbError> {
        let query = format!(
            "ALTER TABLE {} ADD CONSTRAINT fk_{}_{} FOREIGN KEY ({}) REFERENCES {}({})",
            table_name, table_name, column_name, column_name, foreign_table, foreign_column
        );
        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;
        Ok(())
    }
}

#[async_trait]
impl DbClient for PostgresClient {
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
            .map_err(|e| DbError::Transaction(e.to_string()))?; //TODO: check if this is correct
        Ok(Box::new(PostgresTransaction { tx }))
    }

    async fn list_tables(&self) -> Result<Vec<String>, DbError> {
        let query = r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
        "#;
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;

        let tables = rows
            .iter()
            .map(|row| row.try_get::<String, _>("table_name").unwrap_or_default())
            .collect();

        Ok(tables)
    }

    async fn describe_table(&self, table_name: &str) -> Result<TableSchema, DbError> {
        let query = format!(
            r#"
            SELECT column_name, data_type, is_nullable, column_default
            FROM information_schema.columns
            WHERE table_name = '{}'
            "#,
            table_name
        );
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;

        let columns = rows
            .iter()
            .map(|row| ColumnSchema {
                name: row.try_get("column_name").unwrap(),
                data_type: row.try_get("data_type").unwrap(),
                is_nullable: row.try_get::<String, _>("is_nullable").unwrap() == "YES",
                default: row.try_get("column_default").ok(),
            })
            .collect();

        Ok(TableSchema {
            table_name: table_name.to_string(),
            columns,
            indexes: Vec::new(),
        })
    }
}

pub struct PostgresTransaction<'a> {
    tx: sqlx::Transaction<'a, sqlx::Postgres>,
}

#[async_trait]
impl<'a> Transaction for PostgresTransaction<'a> {
    async fn execute(&mut self, query: &str) -> Result<(), DbError> {
        sqlx::query(query)
            .execute(&mut *self.tx)
            .await
            .map_err(|e| DbError::Transaction(e.to_string()))?; // TODO: check if this is correct
        Ok(())
    }

    async fn commit(self: Box<Self>) -> Result<(), DbError> {
        self.tx
            .commit()
            .await
            .map_err(|e| DbError::Transaction(e.to_string())) // TODO: check if this is correct
    }

    async fn rollback(self: Box<Self>) -> Result<(), DbError> {
        self.tx
            .rollback()
            .await
            .map_err(|e| DbError::Transaction(e.to_string())) // TODO: check if this is correct
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
            async fn list_tables(&self) -> Result<Vec<String>, DbError>;
            async fn describe_table(&self, table_name: &str) -> Result<TableSchema, DbError>;
            async fn begin_transaction<'a>(&'a self) -> Result<Box<dyn Transaction + 'a>, DbError>;
        }
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
            async fn execute(&mut self, query: &str) -> Result<(), DbError>;
            async fn commit(self: Box<Self>) -> Result<(), DbError>;
            async fn rollback(self: Box<Self>) -> Result<(), DbError>;
        }
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        let mut mock_tx = MockTransaction::new();

        mock_tx.expect_commit().returning(|| Ok(()));

        let result = Box::new(mock_tx).commit().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        let mut mock_tx = MockTransaction::new();

        mock_tx.expect_rollback().returning(|| Ok(()));

        let result = Box::new(mock_tx).rollback().await;
        assert!(result.is_ok());
    }
}
