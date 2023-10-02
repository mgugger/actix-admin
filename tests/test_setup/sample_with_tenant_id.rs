use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use actix_admin::prelude::*; 
use std::fmt;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, DeriveActixAdmin, DeriveActixAdminViewModel, DeriveActixAdminModel, DeriveActixAdminModelSelectList)]
#[sea_orm(table_name = "sample_with_tenant_id")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[actix_admin(primary_key)]
    pub id: i32,
    #[actix_admin(searchable)]
    pub title: String,
    #[sea_orm(column_type = "Text")]
    #[actix_admin(searchable, textarea)]
    pub text: String,
    #[actix_admin(tenant_ref)]
    pub tenant_id: i32,
}

impl Display for Model {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
           _ => write!(formatter, "{}", &self.title),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl ActixAdminModelValidationTrait<ActiveModel> for Entity {}

impl ActixAdminModelFilterTrait<Entity> for Entity {}
