//! # Actix Admin
//!
//! The actix-admin crate aims at creating a web admin interface similar to other admin interfaces (such as [flask-admin](https://github.com/flask-admin/flask-admin) in python).
//!
//! See the [documentation](https://mgugger.github.io/actix-admin/) at [https://mgugger.github.io/actix-admin/](https://mgugger.github.io/actix-admin/).

use actix_session::Session;
use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use async_trait::async_trait;
use derive_more::{Display, Error};
use sea_orm::DatabaseConnection;
use serde_derive::Serialize;
use std::fmt::{self, Display as FmtDisplay};
use std::collections::{BTreeMap, HashMap};
use tera::Tera;

pub mod builder;
pub mod csrf;
pub mod model;
pub mod routes;
pub mod tera_templates;
pub mod view_model;

pub mod prelude {
    pub use crate::builder::{ActixAdminBuilder, ActixAdminBuilderTrait};
    pub use crate::csrf::{csrf_token_for, verify_csrf, CsrfError, CSRF_HEADER, CSRF_QUERY_PARAM, CSRF_SESSION_KEY};
    pub use crate::model::{
        ActixAdminModel, ActixAdminModelFilter, ActixAdminModelFilterTrait,
        ActixAdminModelFilterType, ActixAdminModelTrait, ActixAdminModelValidationTrait,
    };
    pub use crate::routes::{bulk_action, create_or_edit_post, get_admin_ctx, SortOrder};
    pub use crate::view_model::{
        ActixAdminBulkAction, ActixAdminFilterOperator, ActixAdminPrimaryKey, ActixAdminViewModel,
        ActixAdminViewModelField, ActixAdminViewModelFieldType, ActixAdminViewModelFilter,
        ActixAdminViewModelParams, ActixAdminViewModelSerializable, ActixAdminViewModelTrait,
    };
    pub use crate::{hashmap, ActixAdminSelectListTrait};
    pub use crate::{ActixAdmin, ActixAdminConfiguration, ActixAdminError, ActixAdminErrorType};
    pub use actix_admin_macros::{
        DeriveActixAdmin, DeriveActixAdminEnumSelectList, DeriveActixAdminModel,
        DeriveActixAdminModelSelectList, DeriveActixAdminViewModel,
    };
    pub use actix_session::Session;
    pub use async_trait::async_trait;
}

use crate::prelude::*;

#[macro_export]
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key.to_string(), $val); )*
         map
    }}
}

// SelectListTrait
#[async_trait]
pub trait ActixAdminSelectListTrait {
    async fn get_key_value(
        db: &DatabaseConnection,
        tenant_ref: Option<i32>
    ) -> core::result::Result<Vec<(String, String)>, ActixAdminError>;
}

#[derive(Clone)]
pub struct ActixAdminConfiguration {
    pub enable_auth: bool,
    pub user_is_logged_in: Option<for<'a> fn(&'a Session) -> bool>,
    pub user_tenant_ref: Option<for<'a> fn(&'a Session) -> Option<i32>>,
    pub login_link: Option<String>,
    pub logout_link: Option<String>,
    pub file_upload_directory: &'static str,
    pub navbar_title: &'static str,
    pub base_path: &'static str,
    pub custom_css_paths: Option<Vec<String>>,
    pub custom_js_paths: Option<Vec<String>>,
    /// When `true` (default), every state-changing route (POST/DELETE/PUT) is
    /// gated by a CSRF token stored in the actix-session cookie. Templates
    /// automatically wire the token into every HTMX request as the
    /// `X-CSRF-Token` header, and inject a hidden `_csrf` input into forms.
    ///
    /// Requires an `actix-session` middleware to be installed. If your admin
    /// deployment is behind a non-cookie-session auth flow and you do not want
    /// this protection (e.g. tests, an isolated intranet), set to `false`.
    pub enable_csrf: bool,
}

impl Default for ActixAdminConfiguration {
    fn default() -> Self {
        Self {
            enable_auth: false,
            user_is_logged_in: None,
            user_tenant_ref: None,
            login_link: None,
            logout_link: None,
            file_upload_directory: "./file_uploads",
            navbar_title: "Actix Admin",
            base_path: "/admin",
            custom_css_paths: None,
            custom_js_paths: None,
            enable_csrf: true,
        }
    }
}

#[derive(Clone)]
pub struct ActixAdmin {
    pub entity_names: BTreeMap<String, Vec<ActixAdminMenuElement>>,
    pub view_models: HashMap<String, ActixAdminViewModel>,
    pub card_grids: HashMap<String, Vec<Vec<String>>>,
    pub configuration: ActixAdminConfiguration,
    pub tera: Tera,
    pub support_path: Option<String>
}

#[derive(PartialEq, Eq, Clone, Serialize)]
pub struct ActixAdminMenuElement {
    pub name: String,
    pub link: String,
    pub is_custom_handler: bool,
}

#[derive(Debug, Error)]
pub struct ActixAdminError {
    pub ty: ActixAdminErrorType,
    pub msg: String,
}

impl FmtDisplay for ActixAdminError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}: {}", &self.ty, &self.msg)
    }
}

// Errors
#[derive(Debug, Display, Error, PartialEq, Eq)]
pub enum ActixAdminErrorType {
    #[display("Internal error")]
    InternalError,

    #[display("Form has validation errors")]
    ValidationErrors,

    #[display("Bad request")]
    BadRequest,

    #[display("Unauthorized")]
    Unauthorized,

    #[display("Forbidden")]
    Forbidden,

    #[display("Could not list entities")]
    ListError,

    #[display("Could not create entity")]
    CreateError,

    #[display("Could not delete entity")]
    DeleteError,

    #[display("Could not edit entity")]
    EditError,

    #[display("Database error")]
    DatabaseError,

    #[display("Entity does not exist")]
    EntityDoesNotExistError,

    #[display("Upload error")]
    UploadError,

    #[display("IO error")]
    IoError,

    #[display("CSRF token missing or invalid")]
    CsrfError,

    #[display("Unknown bulk action")]
    UnknownBulkAction,
}

impl ActixAdminError {
    pub fn new(ty: ActixAdminErrorType, msg: impl Into<String>) -> Self {
        Self { ty, msg: msg.into() }
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(ActixAdminErrorType::BadRequest, msg)
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(ActixAdminErrorType::EntityDoesNotExistError, msg)
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(ActixAdminErrorType::InternalError, msg)
    }
}

impl error::ResponseError for ActixAdminError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        use ActixAdminErrorType::*;
        match self.ty {
            BadRequest | ValidationErrors => StatusCode::BAD_REQUEST,
            Unauthorized => StatusCode::UNAUTHORIZED,
            Forbidden | CsrfError => StatusCode::FORBIDDEN,
            EntityDoesNotExistError | UnknownBulkAction => StatusCode::NOT_FOUND,
            InternalError | ListError | CreateError | DeleteError | EditError
            | DatabaseError | UploadError | IoError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

macro_rules! impl_from_error {
    ($($err:ty => $ty:ident),* $(,)?) => {
        $(
            impl From<$err> for ActixAdminError {
                fn from(err: $err) -> Self {
                    Self { ty: ActixAdminErrorType::$ty, msg: err.to_string() }
                }
            }
        )*
    };
}

impl_from_error! {
    sea_orm::DbErr => DatabaseError,
    std::io::Error => IoError,
    actix_multipart::MultipartError => UploadError,
    serde_urlencoded::de::Error => BadRequest,
}

// Notifications
#[derive(Debug, Display, Serialize)]
pub enum ActixAdminNotificationType {
    #[display("is-danger")]
    Danger,
}

#[derive(Debug, Serialize)]
pub struct ActixAdminNotification {
    css_class: String,
    message: String,
}

impl From<ActixAdminError> for ActixAdminNotification {
    fn from(e: ActixAdminError) -> Self {
        Self {
            css_class: ActixAdminNotificationType::Danger.to_string(),
            message: e.to_string(),
        }
    }
}
