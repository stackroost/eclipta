use clap::Args;
use crate::utils::db::ensure_db_ready;
use crate::utils::logger::{success, error};

#[derive(Args, Debug)]
pub struct CheckDbOptions {
    #[arg(long, default_value = "false")]
    pub verbose: bool,
}

pub async fn handle_check_db(_opts: CheckDbOptions) -> Result<(), Box<dyn std::error::Error>> {
    match ensure_db_ready().await {
        Ok(_) => {
            success("Database connection successful & migrations applied!");
            Ok(())
        }
        Err(e) => {
            error(&format!("Database check failed: {}", e));
            Err(e)
        }
    }
}
