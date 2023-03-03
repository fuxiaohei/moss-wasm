use crate::entity::function_info;
use crate::entity::prelude::FunctionInfo;
use crate::DB;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::NotSet;
use sea_orm::ActiveValue::Set;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::TryIntoModel;
use tracing::debug;

/// upsert_info
#[tracing::instrument(skip(function_model))]
pub async fn upsert_info(
    function_model: function_info::Model,
) -> Result<function_info::Model, crate::Error> {
    let db = DB.get().unwrap();

    // get function info by name and user id
    let function_info = FunctionInfo::find()
        .filter(function_info::Column::Name.contains(&function_model.name))
        .filter(function_info::Column::UserId.eq(function_model.user_id))
        .one(db)
        .await?;

    let mut active_model: function_info::ActiveModel;

    // if record is found, set id
    if function_info.is_some() {
        debug!(
            "function is found, update it, id: {}, user_id: {}, name: {}",
            function_model.id, function_model.user_id, function_model.name,
        );
        active_model =
            <function_info::Model as Into<function_info::ActiveModel>>::into(function_model)
                .reset_all();
        active_model = active_model.reset_all();
        active_model.id = Set(function_info.as_ref().unwrap().id);
        active_model.not_set(function_info::Column::CreatedAt);
        active_model.not_set(function_info::Column::Uuid);
    } else {
        debug!(
            "function is not found, create it, user_id: {}, name: {}",
            function_model.user_id, function_model.name,
        );
        active_model =
            <function_info::Model as Into<function_info::ActiveModel>>::into(function_model)
                .reset_all();
        active_model.id = NotSet;
        active_model.uuid = Set(uuid::Uuid::new_v4().to_string());
    }

    let result = active_model
        .save(db)
        .await
        .map_err(crate::Error::DatabaseError)?;
    Ok(result.try_into_model().unwrap())
}
