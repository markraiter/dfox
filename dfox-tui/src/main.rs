use std::sync::Arc;

use dfox_lib::DbManager;
use ui::DatabaseClientUI;
mod db;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_manager = Arc::new(DbManager::new());
    let mut tui = DatabaseClientUI::new(db_manager);
    tui.run_ui().await?;

    Ok(())
}
