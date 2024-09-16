use dfox::client::tui::DatabaseClientUI;
use dfox::DbManager;
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let db_manager = Arc::new(DbManager::new());

    let config = dfox::models::connections::ConnectionConfig {
        db_type: dfox::models::connections::DbType::Postgres,
        database_url: env::var("DATABASE_URL").expect("must be set").to_string(),
    };

    db_manager.add_connection(config).await?;

    let tui = DatabaseClientUI::new(db_manager);
    tui.run().await?;

    Ok(())
}

