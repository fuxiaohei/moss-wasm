use crate::entity::prelude::UserToken;
use crate::entity::user_token;
use moss_storage::db;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;

pub async fn get_user_token(token: String) -> Result<user_token::Model, crate::Error> {
    let db = db::DB.get().unwrap();
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
