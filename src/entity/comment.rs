use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use actix_admin::{ DeriveActixAdminModel };

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, DeriveActixAdminModel)]
#[sea_orm(table_name = "comment")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub text: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}


impl From<Entity> for ActixAdminModel {
    fn from(entity: Entity) -> Self {
        ActixAdminModel {
            fields: Vec::new()
        }
    }
}

#[async_trait]
impl ActixAdminModelTrait for Entity {
    async fn list(&self, db: &DatabaseConnection, page: usize, posts_per_page: usize) -> Vec<&str> {
        use sea_orm::{ query::* };
        let paginator = Entity::find()
            .order_by_asc(Column::Id)
            .paginate(db, posts_per_page);
        let entities = paginator
            .fetch_page(page - 1)
            .await
            .expect("could not retrieve entities");
        //entities to ActixAdminModel
        vec![

        ]
    }
}