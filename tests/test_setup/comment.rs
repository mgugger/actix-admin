use std::fmt::{self, Display};

use super::{post, Post};
use actix_admin::prelude::*;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Debug,
    PartialEq,
    DeriveEntityModel,
    Deserialize,
    Serialize,
    DeriveActixAdmin,
    DeriveActixAdminModel,
    DeriveActixAdminViewModel,
)]
#[sea_orm(table_name = "comment")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[actix_admin(primary_key)]
    pub id: i32,
    #[actix_admin(searchable)]
    pub comment: String,
    #[sea_orm(column_type = "Text")]
    #[actix_admin(html_input_type = "email")]
    pub user: String,
    #[sea_orm(column_type = "DateTime")]
    pub insert_date: DateTime,
    pub is_visible: bool,
    #[actix_admin(select_list = "Post", foreign_key = "Post", use_tom_select_callback)]
    pub post_id: Option<i32>,
    #[actix_admin(ceil = 2)]
    pub my_decimal: Decimal,
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
            errors.insert(
                "my_decimal".to_string(),
                "Must be larger than 100".to_string(),
            );
        }
        errors
    }
}

impl ActixAdminModelFilterTrait<Entity> for Entity {
    fn get_filter() -> Vec<ActixAdminModelFilter<Entity>> {
        vec![
            // Operator-aware filter used by the new-features integration test
            // to assert that the `filter_<name>__op=` query param is
            // rendered and accepted.
            ActixAdminModelFilter::new(
                "User",
                ActixAdminModelFilterType::Text,
                |q: sea_orm::Select<Entity>, v| -> sea_orm::Select<Entity> {
                    q.apply_if(v, |query, val: String| query.filter(Column::User.eq(val)))
                },
            )
            .with_operators(vec![
                ActixAdminFilterOperator::Contains,
                ActixAdminFilterOperator::Equals,
                ActixAdminFilterOperator::NotEquals,
                ActixAdminFilterOperator::IsNull,
            ])
            .with_operator_filter(
                |q: sea_orm::Select<Entity>, v, op| -> sea_orm::Select<Entity> {
                    use ActixAdminFilterOperator::*;
                    match (v, op) {
                        (_, Some(IsNull)) => q.filter(Column::User.eq("")),
                        (Some(val), Some(NotEquals)) => q.filter(Column::User.ne(val)),
                        (Some(val), Some(Contains)) => q.filter(Column::User.contains(&val)),
                        (Some(val), _) => q.filter(Column::User.eq(val)),
                        _ => q,
                    }
                },
            ),
        ]
    }
}

impl Display for Model {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            _ => write!(formatter, "{} {}", &self.insert_date, &self.user),
        }
    }
}
