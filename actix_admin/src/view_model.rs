use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use serde::{Serialize};

use crate::ActixAdminModel;

#[async_trait(?Send)]
pub trait ActixAdminViewModelTrait {
    async fn list(
        db: &DatabaseConnection,
        page: usize,
        entities_per_page: usize,
    ) -> (usize, Vec<ActixAdminModel>);
    
    // TODO: Replace return value with proper Result Type containing Ok or Err
    async fn create_entity(db: &DatabaseConnection, model: ActixAdminModel) -> ActixAdminModel;
    async fn delete_entity(db: &DatabaseConnection, id: i32) -> bool;
    async fn get_entity(db: &DatabaseConnection, id: i32) -> ActixAdminModel;
    async fn edit_entity(db: &DatabaseConnection, id: i32, model: ActixAdminModel) -> ActixAdminModel;
    
    fn get_entity_name() -> String;
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminViewModel {
    pub entity_name: String,
    pub primary_key: String,
    pub fields: Vec<(String, String)>,
}