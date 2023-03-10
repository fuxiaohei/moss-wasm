//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "function_resource")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    #[sea_orm(unique)]
    pub name: String,
    pub cpu_time: i32,
    pub memory_usage: i32,
    pub wall_time: i32,
    pub fetch_counts: i32,
    pub fetch_remote_list: String,
    pub status: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::function_info::Entity")]
    FunctionInfo,
}

impl Related<super::function_info::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FunctionInfo.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
