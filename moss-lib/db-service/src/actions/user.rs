use std::time;

use crate::entity::prelude::UserToken;
use crate::entity::user_token;
use crate::DB;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;

/// find_by_token finds a user token by token
pub async fn find_by_token(token: String) -> Result<user_token::Model, crate::Error> {
    let db = DB.get().unwrap();
    let user_token = UserToken::find()
        .filter(user_token::Column::AccessToken.contains(&token))
        .filter(user_token::Column::Status.contains("active"))
        .filter(user_token::Column::From.contains("moss-cli"))
        .one(db)
        .await
        .map_err(crate::Error::DatabaseError)?;
    if user_token.is_none() {
        return Err(crate::Error::RecordNotFound);
    }
    Ok(user_token.unwrap())
}

/// is_token_expired checks if a token is expired
pub fn is_token_expired(token: &user_token::Model) -> bool {
    let now_unixstamp = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    token.expired_at < now_unixstamp as i32
}
