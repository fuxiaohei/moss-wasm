#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Record not found")]
    RecordNotFound,

    #[error("Record already exists: '{0}' is already taken by '{1}'")]
    RecordExists(String, String),

    #[error("Record is invalid: {0}")]
    RecordStatusInvalid(String),

    #[error("Database error: {0}")]
    DbInternal(#[from] sea_orm::error::DbErr),

    /// TokenExpired means token is expired
    #[error("Token expired")]
    TokenExpired,
    /// TokenNotFound means token is not found
    #[error("Token not found")]
    TokenNotFound,
    /// TokenInvalidUsage means wrong usage
    #[error("Token invalid from")]
    TokenInvalidUsage,
    /// TokenInactive means this token is disabled
    #[error("Token inactive")]
    TokenInactive,
    /// TokenSecretIncorrect
    #[error("Token secret incorrect")]
    TokenSecretIncorrect,

    /// UserNotFound
    #[error("User not found")]
    UserNotFound,
    /// UserInActive
    #[error("User inactive")]
    UserInactive,
}
