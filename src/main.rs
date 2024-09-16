use dfox::client::tui::DatabaseClientUI;
use dfox::DbManager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_manager = Arc::new(DbManager::new());
    let mut tui = DatabaseClientUI::new(db_manager);
    tui.run().await?;

    Ok(())
}
