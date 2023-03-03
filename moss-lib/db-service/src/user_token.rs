use crate::entity::prelude::{UserInfo, UserToken};
use crate::entity::{user_info, user_token};
use crate::errors::Error;
use crate::DB;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use std::time;

/// UserTokenBundle contains token and user info
pub struct UserTokenBundle {
    pub info: user_info::Model,
    pub token: user_token::Model,
}

impl UserTokenBundle {
    pub fn validate_signature(&self, sign: String) -> Result<(), Error> {
        if self.token.secret_token != sign {
            return Err(Error::TokenSecretIncorrect);
        }
        Ok(())
    }
}

/// get gets token
pub async fn get(api_key: String, usage: &str) -> Result<user_token::Model, Error> {
    let db = DB.get().unwrap();
    let user_token = UserToken::find()
        .filter(user_token::Column::AccessToken.contains(&api_key))
        .one(db)
        .await
        .map_err(Error::DbInternal)?;
    let user_token = user_token.ok_or(Error::TokenNotFound)?;
    validate_token(&user_token, usage)?;
    Ok(user_token)
}

/// verify token
pub async fn verify(
    api_key: String,
    api_secret: String,
    usage: &str,
) -> Result<UserTokenBundle, Error> {
    let db = DB.get().unwrap();
    let user_token = UserToken::find()
        .filter(user_token::Column::AccessToken.contains(&api_key))
        .one(db)
        .await
        .map_err(Error::DbInternal)?;
    let user_token = user_token.ok_or(Error::TokenNotFound)?;
    validate_token(&user_token, usage)?;

    let user_info = user_token
        .find_related(UserInfo)
        .one(db)
        .await
        .map_err(Error::DbInternal)?;
    let user_info = user_info.ok_or(Error::UserNotFound)?;
    if user_info.status != "active" {
        return Err(Error::UserInactive);
    }
    let bundle = UserTokenBundle {
        info: user_info,
        token: user_token,
    };
    bundle.validate_signature(api_secret)?;
    Ok(bundle)
}

/// is_token_expired checks if a token is expired
fn validate_token(token: &user_token::Model, usage: &str) -> Result<(), Error> {
    if token.status != "active" {
        return Err(Error::TokenInactive);
    }
    if token.from != usage {
        return Err(Error::TokenInvalidUsage);
    }
    let now_unixstamp = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if token.expired_at > 0 && token.expired_at < now_unixstamp as i32 {
        return Err(Error::TokenExpired);
    }
    Ok(())
}
