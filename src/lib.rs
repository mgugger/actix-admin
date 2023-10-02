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
use tera::Tera;
use std::collections::HashMap;

pub mod builder;
pub mod model;
pub mod routes;
pub mod view_model;
pub mod tera_templates;

pub mod prelude {
    pub use crate::builder::{ActixAdminBuilder, ActixAdminBuilderTrait};
    pub use crate::model::{ActixAdminModel, ActixAdminModelTrait, ActixAdminModelValidationTrait, ActixAdminModelFilter, ActixAdminModelFilterTrait, ActixAdminModelFilterType};
    pub use crate::routes::{create_or_edit_post, get_admin_ctx, SortOrder};
    pub use crate::view_model::{
        ActixAdminViewModel, ActixAdminViewModelParams, ActixAdminViewModelField, ActixAdminViewModelFieldType,
        ActixAdminViewModelSerializable, ActixAdminViewModelTrait, ActixAdminViewModelFilter
    };
    pub use crate::{hashmap, ActixAdminSelectListTrait};
    pub use crate::{ActixAdmin, ActixAdminConfiguration, ActixAdminError};
    pub use actix_admin_macros::{
        DeriveActixAdmin, DeriveActixAdminEnumSelectList, DeriveActixAdminModel,
        DeriveActixAdminModelSelectList, DeriveActixAdminViewModel,
    };
    pub use actix_session::Session;
    pub use async_trait::async_trait;
    pub use itertools::izip;
    pub use lazy_static::lazy_static;
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
    pub navbar_title: &'static str
}

#[derive(Clone)]
pub struct ActixAdmin {
    pub entity_names: HashMap<String, Vec<ActixAdminMenuElement>>,
    pub view_models: HashMap<String, ActixAdminViewModel>,
    pub configuration: ActixAdminConfiguration,
    pub tera: Tera
}

#[derive(PartialEq, Eq, Clone, Serialize)]
pub struct ActixAdminMenuElement {
    pub name: String,
    pub link: String,
    pub is_custom_handler: bool,
}

// Errors
#[derive(Debug, Display, Error)]
pub enum ActixAdminError {
    #[display(fmt = "Internal error")]
    InternalError,

    #[display(fmt = "Form has validation errors")]
    ValidationErrors,

    #[display(fmt = "Could not list entities")]
    ListError,

    #[display(fmt = "Could not create entity")]
    CreateError,

    #[display(fmt = "Could not delete entity")]
    DeleteError,

    #[display(fmt = "Could not edit entity")]
    EditError,

    #[display(fmt = "Database error")]
    DatabaseError,

    #[display(fmt = "Entity does not exist")]
    EntityDoesNotExistError,
}

impl error::ResponseError for ActixAdminError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::convert::From<sea_orm::DbErr> for ActixAdminError {
    fn from(_err: sea_orm::DbErr) -> ActixAdminError {
        ActixAdminError::DatabaseError
    }
}

// Notifications
#[derive(Debug, Display, Serialize)]
pub enum ActixAdminNotificationType {
    #[display(fmt = "is-danger")]
    Danger,
}

#[derive(Debug, Serialize)]
pub struct ActixAdminNotification {
    css_class: String,
    message: String,
}

impl std::convert::From<ActixAdminError> for ActixAdminNotification {
    fn from(e: ActixAdminError) -> ActixAdminNotification {
        ActixAdminNotification {
            css_class: ActixAdminNotificationType::Danger.to_string(),
            message: e.to_string(),
        }
    }
}
