pub mod entity;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Record not found")]
    RecordNotFound,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::error::DbErr),
}

mod user_token;
pub use user_token::get_user_token;
