use async_trait::async_trait;
use regex::Regex;
use sea_orm::DatabaseConnection;
use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::{ActixAdminModel, SortOrder, model::ActixAdminModelFilterType};
use actix_session::Session;
use std::convert::From;
use crate::ActixAdminError;
pub struct ActixAdminViewModelParams {
    pub page: Option<u64>,
    pub entities_per_page: Option<u64>,
    pub viewmodel_filter: Vec<ActixAdminViewModelFilter>,
    pub search: String,
    pub sort_by: String,
    pub sort_order: SortOrder,
    pub tenant_ref: Option<i32>
}

#[async_trait(?Send)]
pub trait ActixAdminViewModelTrait {
    async fn list(
        db: &DatabaseConnection,
        params: &ActixAdminViewModelParams
    ) -> Result<(Option<u64>, Vec<ActixAdminModel>), ActixAdminError>;
    
    // TODO: Replace return value with proper Result Type containing Ok or Err
    async fn create_entity(db: &DatabaseConnection, model: ActixAdminModel, tenant_ref: Option<i32>) -> Result<ActixAdminModel, ActixAdminError>;
    async fn delete_entity(db: &DatabaseConnection, id: i32, tenant_ref: Option<i32>) -> Result<bool, ActixAdminError>;
    async fn get_entity(db: &DatabaseConnection, id: i32, tenant_ref: Option<i32>) -> Result<ActixAdminModel, ActixAdminError>;
    async fn edit_entity(db: &DatabaseConnection, id: i32, model: ActixAdminModel, tenant_ref: Option<i32>) -> Result<ActixAdminModel, ActixAdminError>;
    async fn get_select_lists(db: &DatabaseConnection, tenant_ref: Option<i32>) -> Result<HashMap<String, Vec<(String, String)>>, ActixAdminError>;
    async fn get_viewmodel_filter(db: &DatabaseConnection) -> HashMap<String, ActixAdminViewModelFilter>;
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
    pub user_can_access: Option<fn(&Session) -> bool>,
    pub default_show_aside: bool
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminViewModelSerializable {
    pub entity_name: String,
    pub primary_key: String,
    pub fields: &'static [ActixAdminViewModelField],
    pub show_search: bool,
    pub default_show_aside: bool
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminViewModelFilter {
    pub name: String,
    pub value: Option<String>,
    pub values: Option<Vec<(String, String)>>,
    pub filter_type: Option<ActixAdminModelFilterType>
}

// TODO: better alternative to serialize only specific fields for ActixAdminViewModel
impl From<ActixAdminViewModel> for ActixAdminViewModelSerializable {
    fn from(entity: ActixAdminViewModel) -> Self {
        ActixAdminViewModelSerializable {
            entity_name: entity.entity_name,
            primary_key: entity.primary_key,
            fields: entity.fields,
            show_search: entity.show_search,
            default_show_aside: entity.default_show_aside
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
    pub field_type: ActixAdminViewModelFieldType,
    pub list_sort_position: usize,
    pub list_hide_column: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pub list_regex_mask: Option<Regex>,
    pub foreign_key: String,
    pub is_tenant_ref: bool
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