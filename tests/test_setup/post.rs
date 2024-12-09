use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use actix_admin::prelude::*; 
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, DeriveActixAdmin, DeriveActixAdminViewModel, DeriveActixAdminModel, DeriveActixAdminModelSelectList)]
#[sea_orm(table_name = "post")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[actix_admin(primary_key)]
    pub id: i32,
    #[actix_admin(searchable)]
    pub title: String,
    #[sea_orm(column_type = "Text")]
    #[actix_admin(shorten="100", searchable, textarea)]
    pub text: String,
    #[actix_admin(select_list="Tea")]
    pub tea_mandatory: Tea,
    #[actix_admin(select_list="Tea")]
    pub tea_optional: Option<Tea>,
    pub insert_date: Date,
}

impl Display for Model {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
           _ => write!(formatter, "{}", &self.title),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::comment::Entity")]
    Comment,
}

impl Related<super::comment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Comment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveDisplay, DeriveActiveEnum, Deserialize, Serialize, DeriveActixAdminEnumSelectList)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tea")]
pub enum Tea {
    #[sea_orm(string_value = "EverydayTea")]
    EverydayTea,
    #[sea_orm(string_value = "BreakfastTea")]
    BreakfastTea,
}

impl FromStr for Tea {
    type Err = ();

    fn from_str(input: &str) -> Result<Tea, Self::Err> {
        match input {
            "EverydayTea"  => Ok(Tea::EverydayTea),
            "BreakfastTea"  => Ok(Tea::BreakfastTea),
            _      => Err(()),
        }
    }
}

impl ActixAdminModelValidationTrait<ActiveModel> for Entity {}

impl ActixAdminModelFilterTrait<Entity> for Entity {}
