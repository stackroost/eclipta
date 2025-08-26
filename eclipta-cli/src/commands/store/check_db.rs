use crate::utils::db::init_db;
use crate::utils::logger::{success, error};

#[derive(clap::Args)]
pub struct CheckDbOptions {}

pub async fn handle_check_db(_opts: CheckDbOptions) -> Result<(), Box<dyn std::error::Error>> {
    match init_db().await {
        Ok(_) => {
            success("Database connection successful & migrations applied!");
            Ok(())
        }
        Err(e) => {
            error(&format!("Database connection failed: {}", e));
            Err(Box::new(e))
        }
    }
}
