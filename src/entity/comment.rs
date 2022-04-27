use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use actix_admin::{ DeriveActixAdminModel };

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, DeriveActixAdminModel)]
#[sea_orm(table_name = "comment")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub comment: String,
    #[sea_orm(column_type = "Text")]
    pub user: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}