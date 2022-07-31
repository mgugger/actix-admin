use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use serde::{Serialize};
use std::collections::HashMap;
use crate::ActixAdminModel;

#[async_trait(?Send)]
pub trait ActixAdminViewModelTrait {
    async fn list(
        db: &DatabaseConnection,
        page: usize,
        entities_per_page: usize,
        search: &String
    ) -> (usize, Vec<ActixAdminModel>);
    
    // TODO: Replace return value with proper Result Type containing Ok or Err
    async fn create_entity(db: &DatabaseConnection, model: ActixAdminModel) -> ActixAdminModel;
    async fn delete_entity(db: &DatabaseConnection, id: i32) -> bool;
    async fn get_entity(db: &DatabaseConnection, id: i32) -> ActixAdminModel;
    async fn edit_entity(db: &DatabaseConnection, id: i32, model: ActixAdminModel) -> ActixAdminModel;
    async fn get_select_lists(db: &DatabaseConnection) -> HashMap<String, Vec<(String, String)>>;

    fn get_entity_name() -> String;

    fn get_list_link(entity_name: &String) -> String {
        format!("/admin/{}/list", entity_name)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminViewModel {
    pub entity_name: String,
    pub primary_key: String,
    pub fields: Vec<ActixAdminViewModelField>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminViewModelField {
    pub field_name: String,
    pub html_input_type: String,
    pub select_list: String,
    pub is_option: bool
}