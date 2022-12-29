//! # Actix Admin
//!
//! The actix-admin crate aims at creating a web admin interface similar to other admin interfaces (such as [flask-admin](https://github.com/flask-admin/flask-admin) in python).
//!
//! See the [documentation](https://mgugger.github.io/actix-admin/) at [https://mgugger.github.io/actix-admin/](https://mgugger.github.io/actix-admin/).

use lazy_static::lazy_static;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use std::collections::HashMap;
use tera::{Tera, Result, to_value, try_get_value };
use std::{ hash::BuildHasher};
use actix_session::{Session};
use async_trait::async_trait;
use derive_more::{Display, Error};
use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};

pub mod view_model;
pub mod model;
pub mod routes;
pub mod builder;

pub mod prelude {
    pub use crate::builder::{ ActixAdminBuilder, ActixAdminBuilderTrait};
    pub use crate::model::{ ActixAdminModel, ActixAdminModelValidationTrait, ActixAdminModelTrait};
    pub use crate::view_model::{ ActixAdminViewModel, ActixAdminViewModelTrait, ActixAdminViewModelField, ActixAdminViewModelSerializable, ActixAdminViewModelFieldType };
    pub use actix_admin_macros::{ DeriveActixAdmin, DeriveActixAdminModel, DeriveActixAdminViewModel, DeriveActixAdminEnumSelectList, DeriveActixAdminModelSelectList };
    pub use crate::{ ActixAdminError, ActixAdminAppDataTrait, ActixAdmin, ActixAdminConfiguration };
    pub use crate::{ hashmap, ActixAdminSelectListTrait };
    pub use crate::routes::{ create_or_edit_post, get_admin_ctx };
    pub use crate::{ TERA };
    pub use itertools::izip;
    pub use lazy_static::lazy_static;
    pub use async_trait::async_trait;
    pub use actix_session::{Session};
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

// globals
lazy_static! {
    pub static ref TERA: Tera = {
       let mut tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
       tera.register_filter("get_html_input_type", get_html_input_type);
       tera.register_filter("get_html_input_class", get_html_input_class);
       tera.register_filter("get_icon", get_icon);
       tera
    };
}

pub fn get_html_input_class<S: BuildHasher>(value: &tera::Value, _: &HashMap<String, tera::Value, S>) -> Result<tera::Value> {
    let field = try_get_value!("get_html_input_class", "value", ActixAdminViewModelField, value);
    let html_input_type = match field.field_type {
        ActixAdminViewModelFieldType::TextArea => "textarea",
        ActixAdminViewModelFieldType::Checkbox => "checkbox",
        _ => "input"
    };

    Ok(to_value(html_input_type).unwrap())
}

pub fn get_icon<S: BuildHasher>(value: &tera::Value, _: &HashMap<String, tera::Value, S>) -> Result<tera::Value> {
    let field = try_get_value!("get_icon", "value", String, value);
    let font_awesome_icon = match field.as_str() {
        "true" => "<i class=\"fa-solid fa-check\"></i>",
        "false" => "<i class=\"fa-solid fa-xmark\"></i>",
        _ => panic!("not implemented icon")
    };

    Ok(to_value(font_awesome_icon).unwrap())
}

pub fn get_html_input_type<S: BuildHasher>(value: &tera::Value, _: &HashMap<String, tera::Value, S>) -> Result<tera::Value> {
    let field = try_get_value!("get_html_input_type", "value", ActixAdminViewModelField, value);

    // TODO: convert to option
    if field.html_input_type != "" {
        return Ok(to_value(field.html_input_type).unwrap())
    }

    let html_input_type = match field.field_type {
        ActixAdminViewModelFieldType::Text => "text",
        ActixAdminViewModelFieldType::DateTime => "datetime-local",
        ActixAdminViewModelFieldType::Date => "date",
        ActixAdminViewModelFieldType::Checkbox => "checkbox",
        ActixAdminViewModelFieldType::FileUpload => "file",
        _ => "text"
    };

    Ok(to_value(html_input_type).unwrap())
}

// AppDataTrait
pub trait ActixAdminAppDataTrait {
    fn get_db(&self) -> &DatabaseConnection;
    fn get_actix_admin(&self) -> &ActixAdmin;
}

// SelectListTrait
#[async_trait]
pub trait ActixAdminSelectListTrait {
    async fn get_key_value(db: &DatabaseConnection) -> core::result::Result<Vec<(String, String)>, ActixAdminError>;
}


#[derive(Clone)]
pub struct ActixAdminConfiguration {
    pub enable_auth: bool,
    pub user_is_logged_in: Option<for<'a> fn(&'a Session) -> bool>,
    pub login_link: Option<String>,
    pub logout_link: Option<String>,
    pub file_upload_directory: &'static str
}

#[derive(Clone)]
pub struct ActixAdmin {
    pub entity_names: HashMap<String, Vec<ActixAdminMenuElement>>,
    pub view_models: HashMap<String, ActixAdminViewModel>,
    pub configuration: ActixAdminConfiguration
}

#[derive(PartialEq, Eq, Clone, Serialize)]
pub struct ActixAdminMenuElement {
    pub name: String,
    pub link: String,
    pub is_custom_handler: bool
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
    EntityDoesNotExistError
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
    message: String
}

impl std::convert::From<ActixAdminError> for ActixAdminNotification {
    fn from(e: ActixAdminError) -> ActixAdminNotification {
        ActixAdminNotification {
            css_class: ActixAdminNotificationType::Danger.to_string(),
            message: e.to_string()
        }
    }
}