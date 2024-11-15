use std::fmt::Display;
use std::fmt;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use actix_admin::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, DeriveActixAdmin, DeriveActixAdminModel, DeriveActixAdminViewModel)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[actix_admin(primary_key)]
    pub id: i32,
    pub name: String
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActixAdminModelValidationTrait<ActiveModel> for Entity {}

impl ActixAdminModelFilterTrait<Entity> for Entity {}

impl Display for Model {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
           _ => write!(formatter, "{}", &self.name),
        }
    }
}