pub mod entity;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Record not found")]
    RecordNotFound,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::error::DbErr),
}

mod actions;
pub use actions::user_token;

mod config;
pub use config::Config;

mod db;
pub use db::init_db;
pub use db::DB;

