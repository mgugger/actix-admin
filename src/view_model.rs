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

/// Blanket bound for anything usable as an entity primary key in the admin.
///
/// This is what powers the `ActixAdminViewModelTrait::Id` associated type,
/// letting an entity be keyed by `i32`, `i64`, `String`, `uuid::Uuid`, ...
/// as long as the type satisfies the four ubiquitous requirements:
///
/// * `DeserializeOwned` — needed by `actix_web::web::Path<Id>`.
/// * `FromStr`         — needed to parse ids out of form bodies (bulk delete).
/// * `Display`         — needed to render ids into URLs and templates.
/// * `Clone + 'static` — needed by the generated Sea-ORM queries.
pub trait ActixAdminPrimaryKey:
    serde::de::DeserializeOwned + std::str::FromStr + std::fmt::Display + Clone + 'static
{
}
impl<T> ActixAdminPrimaryKey for T where
    T: serde::de::DeserializeOwned + std::str::FromStr + std::fmt::Display + Clone + 'static
{
}

#[async_trait(?Send)]
pub trait ActixAdminViewModelTrait {
    /// The primary-key type of this entity. Defaults to `i32` in the derive
    /// macro output; override by having a `#[actix_admin(primary_key)]` field
    /// with a different type (e.g. `Uuid`, `i64`, `String`).
    type Id: ActixAdminPrimaryKey;

    async fn list(
        db: &DatabaseConnection,
        params: &ActixAdminViewModelParams
    ) -> Result<(Option<u64>, Vec<ActixAdminModel>), ActixAdminError>;
    
    // TODO: Replace return value with proper Result Type containing Ok or Err
    async fn create_entity(db: &DatabaseConnection, model: ActixAdminModel, tenant_ref: Option<i32>) -> Result<ActixAdminModel, ActixAdminError>;
    async fn delete_entity(db: &DatabaseConnection, id: Self::Id, tenant_ref: Option<i32>) -> Result<bool, ActixAdminError>;

    /// Bulk-delete many entities in a single query. Default implementation
    /// falls back to a per-id loop over `delete_entity`, so existing
    /// implementations keep working; the derive-macro override does a single
    /// `DELETE ... WHERE pk IN (...)` query.
    async fn delete_entities(
        db: &DatabaseConnection,
        ids: &[Self::Id],
        tenant_ref: Option<i32>,
    ) -> Result<u64, ActixAdminError> {
        let mut deleted = 0u64;
        for id in ids {
            if Self::delete_entity(db, id.clone(), tenant_ref).await? {
                deleted += 1;
            }
        }
        Ok(deleted)
    }

    async fn get_entity(db: &DatabaseConnection, id: Self::Id, tenant_ref: Option<i32>) -> Result<ActixAdminModel, ActixAdminError>;
    async fn edit_entity(db: &DatabaseConnection, id: Self::Id, model: ActixAdminModel, tenant_ref: Option<i32>) -> Result<ActixAdminModel, ActixAdminError>;
    async fn get_select_lists(db: &DatabaseConnection, tenant_ref: Option<i32>) -> Result<HashMap<String, Vec<(String, String)>>, ActixAdminError>;
    async fn get_viewmodel_filter(db: &DatabaseConnection) -> HashMap<String, ActixAdminViewModelFilter>;
    async fn validate_entity(model: &mut ActixAdminModel, db: &DatabaseConnection);

    fn get_entity_name() -> String;
}

#[derive(Clone)]
pub struct ActixAdminViewModel {
    pub entity_name: String,
    pub primary_key: String,
    pub fields: &'static[ActixAdminViewModelField],
    pub show_search: bool,
    pub user_can_access: Option<fn(&Session) -> bool>,
    pub default_show_aside: bool,
    pub inline_edit: bool
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminViewModelSerializable {
    pub entity_name: String,
    pub primary_key: String,
    pub fields: &'static [ActixAdminViewModelField],
    pub show_search: bool,
    pub default_show_aside: bool,
    pub inline_edit: bool
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminViewModelFilter {
    pub name: String,
    pub value: Option<String>,
    pub foreign_key: Option<String>,
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
            default_show_aside: entity.default_show_aside,
            inline_edit: entity.inline_edit
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
    pub dateformat: Option<String>,
    pub is_option: bool,
    pub field_type: ActixAdminViewModelFieldType,
    pub list_sort_position: usize,
    pub list_hide_column: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pub list_regex_mask: Option<Regex>,
    pub foreign_key: String,
    pub is_tenant_ref: bool,
    pub ceil: Option<u8>,
    pub floor: Option<u8>,
    pub shorten: Option<u16>,
    pub use_tom_select_callback: bool
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