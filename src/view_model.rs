use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::ActixAdminModel;
use actix_session::{Session};
use std::convert::From;
use crate::ActixAdminError;

#[async_trait(?Send)]
pub trait ActixAdminViewModelTrait {
    async fn list(
        db: &DatabaseConnection,
        page: u64,
        entities_per_page: u64,
        search: &String
    ) -> Result<(u64, Vec<ActixAdminModel>), ActixAdminError>;
    
    // TODO: Replace return value with proper Result Type containing Ok or Err
    async fn create_entity(db: &DatabaseConnection, model: ActixAdminModel) -> Result<ActixAdminModel, ActixAdminError>;
    async fn delete_entity(db: &DatabaseConnection, id: i32) -> Result<bool, ActixAdminError>;
    async fn get_entity(db: &DatabaseConnection, id: i32) -> Result<ActixAdminModel, ActixAdminError>;
    async fn edit_entity(db: &DatabaseConnection, id: i32, model: ActixAdminModel) -> Result<ActixAdminModel, ActixAdminError>;
    async fn get_select_lists(db: &DatabaseConnection) -> Result<HashMap<String, Vec<(String, String)>>, ActixAdminError>;
    fn validate_entity(model: &mut ActixAdminModel);

    fn get_entity_name() -> String;

    fn get_base_path(entity_name: &String) -> String {
        format!("/admin/{}", entity_name)
    }
}

#[derive(Clone)]
pub struct ActixAdminViewModel {
    pub entity_name: String,
    pub primary_key: String,
    pub fields: &'static[ActixAdminViewModelField],
    pub show_search: bool,
    pub user_can_access: Option<fn(&Session) -> bool>
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminViewModelSerializable {
    pub entity_name: String,
    pub primary_key: String,
    pub fields: &'static [ActixAdminViewModelField],
    pub show_search: bool
}

// TODO: better alternative to serialize only specific fields for ActixAdminViewModel
impl From<ActixAdminViewModel> for ActixAdminViewModelSerializable {
    fn from(entity: ActixAdminViewModel) -> Self {
        ActixAdminViewModelSerializable {
            entity_name: entity.entity_name,
            primary_key: entity.primary_key,
            fields: entity.fields,
            show_search: entity.show_search
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ActixAdminViewModelFieldType {
    Number,
    Text,
    TextArea,
    Checkbox,
    Date,
    Time,
    DateTime,
    SelectList,
    FileUpload
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActixAdminViewModelField {
    pub field_name: String,
    pub html_input_type: String,
    pub select_list: String,
    pub is_option: bool,
    pub field_type: ActixAdminViewModelFieldType
}

impl ActixAdminViewModelFieldType {
    pub fn get_field_type(type_path: &str, select_list: String, is_textarea: bool, is_file_upload: bool) -> ActixAdminViewModelFieldType {
        if !select_list.is_empty() {
            return ActixAdminViewModelFieldType::SelectList;
        }
        if is_textarea {
            return ActixAdminViewModelFieldType::TextArea;
        }
        if is_file_upload {
            return ActixAdminViewModelFieldType::FileUpload;
        }

        match type_path {
            "i32" => ActixAdminViewModelFieldType::Number,
            "i64" => ActixAdminViewModelFieldType::Number,
            "usize" => ActixAdminViewModelFieldType::Number,
            "String"  => ActixAdminViewModelFieldType::Text,
            "bool"  => ActixAdminViewModelFieldType::Checkbox,
            "DateTimeWithTimeZone" => ActixAdminViewModelFieldType::DateTime,
            "DateTime" => ActixAdminViewModelFieldType::DateTime,
            "Date" => ActixAdminViewModelFieldType::Date,
            _      => ActixAdminViewModelFieldType::Text
        }
    }
}