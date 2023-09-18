use std::fmt::{self, Display};

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use actix_admin::prelude::*;
use super::{Post, post};
use chrono::NaiveDateTime;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, DeriveActixAdmin, DeriveActixAdminModel, DeriveActixAdminViewModel)]
#[sea_orm(table_name = "comment")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[actix_admin(primary_key)]
    pub id: i32,
    
    pub comment: String,
    
    #[sea_orm(column_type = "Text")]
    #[actix_admin(html_input_type = "email", list_regex_mask= "^([a-zA-Z]*)")]
    pub user: String,
    
    #[sea_orm(column_type = "DateTime")]
    pub insert_date: DateTime,
    
    pub is_visible: bool,
    
    #[actix_admin(select_list="Post", foreign_key="Post")]
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

impl Display for Model {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
           _ => write!(formatter, "{} {}", &self.insert_date, &self.user),
        }
    }
}

#[async_trait]
impl ActixAdminModelFilterTrait<Entity> for Entity {
    fn get_filter() -> Vec<ActixAdminModelFilter<Entity>> {
        vec![
            ActixAdminModelFilter::<Entity> {
                name: "User".to_string(),
                filter_type: ActixAdminModelFilterType::Text,
                filter: |q: sea_orm::Select<Entity>, v| -> sea_orm::Select<Entity> {
                    q.apply_if(v, | query, val: String| query.filter(Column::User.eq(val)))
                },
                values: None
            },
            ActixAdminModelFilter::<Entity> {
                name: "Insert Date After".to_string(),
                filter_type: ActixAdminModelFilterType::DateTime,
                filter: |q: sea_orm::Select<Entity>, v| -> sea_orm::Select<Entity> {
                    q.apply_if(v, | query, val: String| query.filter(Column::InsertDate.gte(NaiveDateTime::parse_from_str(&val, "%Y-%m-%dT%H:%M").unwrap())))
                },
                values: None
            },
            ActixAdminModelFilter::<Entity> {
                name: "Is Visible".to_string(),
                filter_type: ActixAdminModelFilterType::Checkbox,
                filter: |q: sea_orm::Select<Entity>, v| -> sea_orm::Select<Entity> {
                    q.apply_if(v, | query, val: String| query.filter(Column::IsVisible.eq(val)))
                },
                values: None
            },
            ActixAdminModelFilter::<Entity> {
                name: "Post".to_string(),
                filter_type: ActixAdminModelFilterType::SelectList,
                filter: |q: sea_orm::Select<Entity>, v| -> sea_orm::Select<Entity> {
                    q.apply_if(v, | query, val: String| query.filter(Column::PostId.eq(val)))
                },
                values: None
            }
        ]
    }

    async fn get_filter_values(filter: &ActixAdminModelFilter<Entity>, db: &DatabaseConnection) -> Option<Vec<(String, String)>> { 
        match filter.name.as_str() {
            "Post" => Some({
                Post::find().order_by_asc(post::Column::Id).all(db).await.unwrap()
                    .iter().map(|p| (p.id.to_string(), p.title.to_string())).collect()
            }),
            _ => None
        }
    }
}

