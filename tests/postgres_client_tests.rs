use dfox::db::postgres::PostgresClient;
use dfox::db::DbClient;
use dfox::models::schema::ColumnSchema;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};
use std::env;
use tokio::fs;

async fn setup_test_db() -> PgPool {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    pool.execute("DROP TABLE IF EXISTS users").await.unwrap();
    pool.execute(
        r#"
            CREATE TABLE users (
                id SERIAL PRIMARY KEY,
                name VARCHAR(100) NOT NULL,
                email VARCHAR(100) NOT NULL
            );
            "#,
    )
    .await
    .unwrap();

    pool
}

#[tokio::test]
async fn test_create_table() {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let client = PostgresClient::connect(&database_url).await.unwrap();

    let columns = vec![
        ColumnSchema {
            name: "id".to_string(),
            data_type: "SERIAL".to_string(),
            is_nullable: false,
            default: None,
        },
        ColumnSchema {
            name: "name".to_string(),
            data_type: "VARCHAR(100)".to_string(),
            is_nullable: false,
            default: None,
        },
        ColumnSchema {
            name: "email".to_string(),
            data_type: "VARCHAR(100)".to_string(),
            is_nullable: false,
            default: None,
        },
    ];

    client.create_table("test_users", &columns).await.unwrap();

    let tables = client.list_tables().await.unwrap();
    assert!(tables.contains(&"test_users".to_string()));
}

// WARN: CHECK THIS TEST!!!
#[tokio::test]
async fn test_import_csv() {
    let pool = setup_test_db().await;
    let client = PostgresClient { pool };

    let file_path = "/tmp/test_import.csv";
    let csv_content = "name,email\nAlice,alice@example.com\nBob,bob@example.com";
    fs::write(file_path, csv_content).await.unwrap();

    client.import_csv("users", file_path).await.unwrap();

    let result = client.query("SELECT name, email FROM users").await.unwrap();
    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn test_export_to_csv() {
    let pool = setup_test_db().await;
    let client = PostgresClient { pool };

    client
            .execute(
                "INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com'), ('Bob', 'bob@example.com');",
            )
            .await
            .unwrap();

    let file_path = "/tmp/test_export.csv";
    client.export_to_csv("users", file_path).await.unwrap();

    let csv_content = fs::read_to_string(file_path).await.unwrap();
    assert!(csv_content.contains("Alice"));
    assert!(csv_content.contains("Bob"));
}

#[tokio::test]
async fn test_create_and_drop_table() {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let client = PostgresClient::connect(&database_url).await.unwrap();

    let columns = vec![
        ColumnSchema {
            name: "id".to_string(),
            data_type: "SERIAL".to_string(),
            is_nullable: false,
            default: None,
        },
        ColumnSchema {
            name: "name".to_string(),
            data_type: "VARCHAR(100)".to_string(),
            is_nullable: false,
            default: None,
        },
    ];

    client.create_table("test_table", &columns).await.unwrap();
    let tables = client.list_tables().await.unwrap();
    assert!(tables.contains(&"test_table".to_string()));

    client.drop_table("test_table").await.unwrap();
    let tables = client.list_tables().await.unwrap();
    assert!(!tables.contains(&"test_table".to_string()));
}

// WARN: CHECK THIS TEST!!!
#[tokio::test]
async fn test_create_and_drop_index() {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let client = PostgresClient::connect(&database_url).await.unwrap();

    client.create_index("users", "email").await.unwrap();

    client.drop_index("idx_users_email").await.unwrap();
}

// WARN: CHECK THIS TEST!!!
#[tokio::test]
async fn test_add_unique_constraint() {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let client = PostgresClient::connect(&database_url).await.unwrap();

    client
        .add_unique_constraint("users", "email")
        .await
        .unwrap();

    let result = client
        .execute("INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com')")
        .await;
    assert!(result.is_err());
}

// WARN: CHECK THIS TEST!!!
#[tokio::test]
async fn test_add_foreign_key() {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let client = PostgresClient::connect(&database_url).await.unwrap();

    let columns = vec![ColumnSchema {
        name: "id".to_string(),
        data_type: "SERIAL".to_string(),
        is_nullable: false,
        default: None,
    }];
    client.create_table("parent_table", &columns).await.unwrap();

    client
        .add_foreign_key("users", "id", "parent_table", "id")
        .await
        .unwrap();
}
