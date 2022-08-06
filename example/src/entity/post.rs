use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use actix_admin::prelude::*; 
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, DeriveActixAdminModel)]
#[sea_orm(table_name = "post")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[actix_admin(primary_key)]
    pub id: i32,
    #[actix_admin(searchable)]
    pub title: String,
    #[sea_orm(column_type = "Text")]
    #[actix_admin(searchable)]
    pub text: String,
    #[actix_admin(select_list="Tea")]
    pub tea_mandatory: Tea,
    #[actix_admin(select_list="Tea")]
    pub tea_optional: Option<Tea>,
    pub insert_date: Date,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Deserialize, Serialize, DeriveActixAdminSelectList)]
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

impl Display for Tea {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Tea::EverydayTea => write!(formatter, "{}", String::from("EverydayTea")),
            Tea::BreakfastTea => write!(formatter, "{}", String::from("BreakfastTea")),
        }
    }
}