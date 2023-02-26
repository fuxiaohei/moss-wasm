use super::config::Config;
use anyhow::Result;
use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use tracing::debug;

pub static DB: OnceCell<DatabaseConnection> = OnceCell::new();

/// init_db init database connection
pub async fn init_db(db_config: &Config) -> Result<()> {
    let url = db_config.url();
    debug!("connect to database: {url}");

    let mut opt = ConnectOptions::new(url);
    opt.max_connections(db_config.pool_size)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(10))
        .max_lifetime(Duration::from_secs(10))
        .sqlx_logging(true);

    let db = Database::connect(opt).await?;
    DB.set(db).unwrap();
    Ok(())
}
