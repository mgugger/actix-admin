use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use actix_admin::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, DeriveActixAdmin, DeriveActixAdminModel, DeriveActixAdminViewModel, DeriveActixAdminViewModelAccess)]
#[sea_orm(table_name = "comment")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[actix_admin(primary_key)]
    pub id: i32,
    pub comment: String,
    #[sea_orm(column_type = "Text")]
    #[actix_admin(html_input_type = "email")]
    pub user: String,
    #[sea_orm(column_type = "DateTime")]
    pub insert_date: DateTime,
    pub is_visible: bool
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}