use async_trait::async_trait;
use regex::Regex;
use sea_orm::DatabaseConnection;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ActixAdminError;
use crate::{model::ActixAdminModelFilterType, ActixAdminModel, SortOrder};
use actix_session::Session;
use std::convert::From;
pub struct ActixAdminViewModelParams {
    pub page: Option<u64>,
    pub entities_per_page: Option<u64>,
    pub viewmodel_filter: Vec<ActixAdminViewModelFilter>,
    pub search: String,
    pub sort_by: String,
    pub sort_order: SortOrder,
    pub tenant_ref: Option<i32>,
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
        params: &ActixAdminViewModelParams,
    ) -> Result<(Option<u64>, Vec<ActixAdminModel>), ActixAdminError>;

    // TODO: Replace return value with proper Result Type containing Ok or Err
    async fn create_entity(
        db: &DatabaseConnection,
        model: ActixAdminModel,
        tenant_ref: Option<i32>,
    ) -> Result<ActixAdminModel, ActixAdminError>;
    async fn delete_entity(
        db: &DatabaseConnection,
        id: Self::Id,
        tenant_ref: Option<i32>,
    ) -> Result<bool, ActixAdminError>;

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

    async fn get_entity(
        db: &DatabaseConnection,
        id: Self::Id,
        tenant_ref: Option<i32>,
    ) -> Result<ActixAdminModel, ActixAdminError>;
    async fn edit_entity(
        db: &DatabaseConnection,
        id: Self::Id,
        model: ActixAdminModel,
        tenant_ref: Option<i32>,
    ) -> Result<ActixAdminModel, ActixAdminError>;
    async fn get_select_lists(
        db: &DatabaseConnection,
        tenant_ref: Option<i32>,
    ) -> Result<HashMap<String, Vec<(String, String)>>, ActixAdminError>;
    async fn get_viewmodel_filter(
        db: &DatabaseConnection,
    ) -> HashMap<String, ActixAdminViewModelFilter>;
    async fn validate_entity(model: &mut ActixAdminModel, db: &DatabaseConnection);

    fn get_entity_name() -> String;
}

/// A user-visible action that can be applied to a selection of rows on the
/// list page ("Archive selected", "Send email", ...). Register via
/// `ActixAdminBuilder::add_bulk_action_for_entity::<E>(...)`.
#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminBulkAction {
    /// URL-safe identifier used as the route segment (`/entity/action/{name}`).
    pub name: String,
    /// Human-readable label rendered in the actions dropdown.
    pub label: String,
    /// Optional Font Awesome icon class, e.g. `"fa-solid fa-archive"`.
    pub icon: Option<String>,
    /// If set, the UI prompts the user with this text before submitting.
    pub confirm: Option<String>,
}

#[derive(Clone)]
pub struct ActixAdminViewModel {
    pub entity_name: String,
    pub primary_key: String,
    pub fields: &'static [ActixAdminViewModelField],
    pub show_search: bool,
    /// Top-level page access. If set and returns `false`, the entity is
    /// invisible and every route 401s. Auth-independent (i.e. also honored
    /// when `enable_auth = false`).
    pub user_can_access: Option<fn(&Session) -> bool>,
    /// Per-action permissions. When `None`, the action defaults to the value of
    /// `user_can_access` (or `true` if that is also `None`).
    pub user_can_create: Option<fn(&Session) -> bool>,
    pub user_can_edit: Option<fn(&Session) -> bool>,
    pub user_can_delete: Option<fn(&Session) -> bool>,
    pub user_can_view_details: Option<fn(&Session) -> bool>,
    pub user_can_export: Option<fn(&Session) -> bool>,
    pub default_show_aside: bool,
    pub inline_edit: bool,
    /// Bulk actions registered for this entity. Cloned into the ViewModel by
    /// the builder when `add_bulk_action_for_entity` is called.
    pub bulk_actions: Vec<ActixAdminBulkAction>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminViewModelSerializable {
    pub entity_name: String,
    pub primary_key: String,
    pub fields: &'static [ActixAdminViewModelField],
    pub show_search: bool,
    pub default_show_aside: bool,
    pub inline_edit: bool,
    /// Serialized permission flags, resolved for the current session. Filled
    /// in per-request by `add_default_context` since the fn hooks themselves
    /// are not serializable.
    #[serde(default)]
    pub can_create: bool,
    #[serde(default)]
    pub can_edit: bool,
    #[serde(default)]
    pub can_delete: bool,
    #[serde(default)]
    pub can_view_details: bool,
    #[serde(default)]
    pub can_export: bool,
    pub bulk_actions: Vec<ActixAdminBulkAction>,
}

/// Comparison operator applied by an advanced filter. Encoded on the wire as
/// `filter_<name>__op=<snake_case_variant>` (case-insensitive).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActixAdminFilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    GreaterEquals,
    LessEquals,
    IsNull,
    IsNotNull,
    InList,
}

impl ActixAdminFilterOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Equals => "equals",
            Self::NotEquals => "not_equals",
            Self::Contains => "contains",
            Self::NotContains => "not_contains",
            Self::GreaterThan => "gt",
            Self::LessThan => "lt",
            Self::GreaterEquals => "gte",
            Self::LessEquals => "lte",
            Self::IsNull => "is_null",
            Self::IsNotNull => "is_not_null",
            Self::InList => "in",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Equals => "=",
            Self::NotEquals => "≠",
            Self::Contains => "contains",
            Self::NotContains => "does not contain",
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::GreaterEquals => "≥",
            Self::LessEquals => "≤",
            Self::IsNull => "is empty",
            Self::IsNotNull => "is not empty",
            Self::InList => "in list",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "equals" | "eq" | "=" => Some(Self::Equals),
            "not_equals" | "ne" | "!=" => Some(Self::NotEquals),
            "contains" | "like" => Some(Self::Contains),
            "not_contains" | "not_like" => Some(Self::NotContains),
            "gt" | ">" => Some(Self::GreaterThan),
            "lt" | "<" => Some(Self::LessThan),
            "gte" | ">=" => Some(Self::GreaterEquals),
            "lte" | "<=" => Some(Self::LessEquals),
            "is_null" | "empty" => Some(Self::IsNull),
            "is_not_null" | "not_empty" => Some(Self::IsNotNull),
            "in" | "in_list" => Some(Self::InList),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminViewModelFilter {
    pub name: String,
    pub value: Option<String>,
    pub foreign_key: Option<String>,
    pub values: Option<Vec<(String, String)>>,
    pub filter_type: Option<ActixAdminModelFilterType>,
    /// Which comparison operators the user may pick from. When empty, no
    /// operator picker is rendered and the filter closure receives
    /// `operator = None` (legacy behavior).
    #[serde(default)]
    pub operators: Vec<ActixAdminFilterOperator>,
    /// The operator selected by the current request, if any.
    #[serde(default)]
    pub operator: Option<ActixAdminFilterOperator>,
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
            inline_edit: entity.inline_edit,
            can_create: true,
            can_edit: true,
            can_delete: true,
            can_view_details: true,
            can_export: true,
            bulk_actions: entity.bulk_actions,
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
    FileUpload,
    /// Rendered as raw HTML in list/show views (value comes from the model as-is).
    /// **Only use this for trusted values** — the field is emitted with `| safe`.
    Html,
    /// A URL that is rendered as an anchor tag in list/show views.
    Url,
    /// An email address that is rendered as a `mailto:` link.
    Email,
    /// A file-upload field whose value is a filename in the entity's upload
    /// directory; rendered as a `<img>` thumbnail in list/show views.
    Image,
    /// A textarea backed by a Markdown WYSIWYG editor (EasyMDE) in the
    /// create/edit form.
    RichText,
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
    pub use_tom_select_callback: bool,
    /// Optional read-only field flag (present but not writable). Read-only
    /// fields are still shown in the show view and in the edit form (disabled).
    #[serde(default)]
    pub readonly: bool,
}

impl ActixAdminViewModelFieldType {
    #[allow(clippy::too_many_arguments)]
    pub fn get_field_type(
        type_path: &str,
        select_list: String,
        is_textarea: bool,
        is_file_upload: bool,
        is_image: bool,
        is_html: bool,
        is_url: bool,
        is_email: bool,
        is_wysiwyg: bool,
    ) -> ActixAdminViewModelFieldType {
        if !select_list.is_empty() {
            return ActixAdminViewModelFieldType::SelectList;
        }
        if is_image {
            return ActixAdminViewModelFieldType::Image;
        }
        if is_wysiwyg {
            return ActixAdminViewModelFieldType::RichText;
        }
        if is_textarea {
            return ActixAdminViewModelFieldType::TextArea;
        }
        if is_file_upload {
            return ActixAdminViewModelFieldType::FileUpload;
        }
        if is_html {
            return ActixAdminViewModelFieldType::Html;
        }
        if is_url {
            return ActixAdminViewModelFieldType::Url;
        }
        if is_email {
            return ActixAdminViewModelFieldType::Email;
        }

        match type_path {
            "i32" => ActixAdminViewModelFieldType::Number,
            "i64" => ActixAdminViewModelFieldType::Number,
            "usize" => ActixAdminViewModelFieldType::Number,
            "String" => ActixAdminViewModelFieldType::Text,
            "bool" => ActixAdminViewModelFieldType::Checkbox,
            "DateTimeWithTimeZone" => ActixAdminViewModelFieldType::DateTime,
            "DateTime" => ActixAdminViewModelFieldType::DateTime,
            "Date" => ActixAdminViewModelFieldType::Date,
            _ => ActixAdminViewModelFieldType::Text,
        }
    }
}
