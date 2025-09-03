use clap::Args;
use crate::utils::db::run_migrations_only;
use crate::utils::logger::{success, error};

#[derive(Args, Debug)]
pub struct MigrateOptions {
    #[arg(long, default_value = "false")]
    pub force: bool,
}

pub async fn handle_migrate(opts: MigrateOptions) -> Result<(), Box<dyn std::error::Error>> {
    if opts.force {
        println!("Force migrating database...");
    } else {
        println!("Running database migrations...");
    }
    
    match run_migrations_only().await {
        Ok(()) => {
            success("Database migrations completed successfully!");
            Ok(())
        }
        Err(e) => {
            error(&format!("Migration failed: {}", e));
            Err(e)
        }
    }
}
