use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use actix_admin::prelude::*;
use super::Post;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, DeriveActixAdmin, DeriveActixAdminModel, DeriveActixAdminViewModel)]
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
    pub is_visible: bool,
    #[actix_admin(select_list="Post")]
    pub post_id: Option<i32>,
    pub my_decimal: Decimal
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::post::Entity",
        from = "Column::PostId",
        to = "super::post::Column::Id"
    )]
    Post,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl ActixAdminModelValidationTrait<ActiveModel> for Entity {
    fn validate(model: &ActiveModel) -> HashMap<String, String> {
        let mut errors = HashMap::new();
        if model.my_decimal.clone().unwrap() < Decimal::from(100 as i16) {
            errors.insert("my_decimal".to_string(), "Must be larger than 100".to_string());
        }
        errors
    }
}