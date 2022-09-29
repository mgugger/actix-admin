//! # Actix Admin
//!
//! The actix-admin crate aims at creating a web admin interface similar to other admin interfaces (such as [flask-admin](https://github.com/flask-admin/flask-admin) in python).
//! 
//! ## Getting Started
//! 
//! * See the [example](https://github.com/mgugger/actix-admin/tree/main/example).
//! * See the step by [step tutorial](https://github.com/mgugger/actix-admin/tree/main/example/StepbyStep.md) 
//! 
//! ## Features
//! 1. Async, builds on [sea-orm](https://crates.io/crates/sea-orm) for the database backend
//! 2. Macros, generate the required implementations for models automatically
//! 3. Authentication, optionally pass authentication handler to implement authentication for views
//! 4. Supports custom validation rules
//! 5. Searchable attributes can be specified
//! 6. Supports a custom index view
//! 
//! ## Screenshot
//! 
//! <img src="https://raw.githubusercontent.com/mgugger/actix-admin/main/static/Screenshot.png"/>
//! 
//! ## Quick overview
//! 
//! ### Required dependencies
//! itertools = "0.10.3"
//! sea-orm = { version = "^0.9.1", features = [ "sqlx-sqlite", "runtime-actix-native-tls", "macros" ], default-features = true }
//! actix_admin = { version = "^0.1.0" }
//! 
//! ### See inlined steps
//! ```
//! use sea_orm::entity::prelude::*;
//! use serde::{Deserialize, Serialize};
//! use actix_admin::prelude::*;
//! use actix_web::web;
//! use actix_web::App;
//! use actix_web::HttpServer;
//! use sea_orm::entity::prelude::*;
//! use sea_orm::entity::prelude::*;
//! use actix_admin::prelude::*;
//! // 1. Import ActixAdmin
//! use actix_admin::prelude::*;
//! 
//! // 2. Use DeriveActixAmin* Macros to implement the traits for the model
//! #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, 
//!     DeriveActixAdmin, DeriveActixAdminModel, DeriveActixAdminViewModel
//! )]
//! #[sea_orm(table_name = "comment")]
//! pub struct Model {
//!     #[sea_orm(primary_key)]
//!     #[serde(skip_deserializing)]
//!     #[actix_admin(primary_key)]
//!     pub id: i32,
//!     pub comment: String
//! }
//! impl ActixAdminModelValidationTrait<ActiveModel> for Entity {}
//! impl ActiveModelBehavior for ActiveModel {}
//! 
//! #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//! pub enum Relation { }
//! 
//! // 3. Add actix-admin to the AppState
//! #[derive(Clone)]
//! pub struct AppState {
//!  pub db: DatabaseConnection,
//!  pub actix_admin: ActixAdmin,
//! }
//! 
//! // 4. Implement the ActixAdminAppDataTrait for the AppState
//! impl ActixAdminAppDataTrait for AppState {
//!     fn get_db(&self) -> &DatabaseConnection {
//!         &self.db
//!     }
//! 
//!     fn get_actix_admin(&self) -> &ActixAdmin {
//!         &self.actix_admin
//!     }
//! }
//!
//! // 5. Setup the actix admin configuration
//! pub fn create_actix_admin_builder() -> ActixAdminBuilder {
//! let comment_view_model = ActixAdminViewModel::from(Entity);
//!
//! let configuration = ActixAdminConfiguration {
//!    enable_auth: false,
//!    user_is_logged_in: None,
//!    login_link: None,
//!    logout_link: None,
//! };
//!
//! let mut admin_builder = ActixAdminBuilder::new(configuration);
//! admin_builder.add_entity::<AppState, Entity>(&comment_view_model);
//!
//! admin_builder
//! }
//! 
//! // 6. Add to the actix app
//! let actix_admin = create_actix_admin_builder().get_actix_admin();
//! //let opt = ConnectOptions::new("sqlite::memory:".to_owned());
//! //let conn = sea_orm::Database::connect(opt).unwrap();
//! //let app_state = AppState {
//! //    db: conn,
//! //    actix_admin: actix_admin,
//! //};
//! 
//! HttpServer::new(move || {
//!     App::new()
//!         //.app_data(web::Data::new(app_state.clone()))
//!         .service(
//!             create_actix_admin_builder().get_scope::<AppState>()
//!         )
//! });
//! ```
//! 
//! ## Access
//! The admin interface will be available under /admin/.

use lazy_static::lazy_static;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use std::collections::HashMap;
use tera::{Tera, Result, to_value, try_get_value };
use std::{ hash::BuildHasher};
use actix_session::{Session};
use async_trait::async_trait;

pub mod view_model;
pub mod model;
pub mod routes;
pub mod builder;

pub mod prelude {
    pub use crate::builder::{ ActixAdminBuilder, ActixAdminBuilderTrait};
    pub use crate::model::{ ActixAdminModel, ActixAdminModelValidationTrait, ActixAdminModelTrait};
    pub use crate::view_model::{ ActixAdminViewModel, ActixAdminViewModelTrait, ActixAdminViewModelField, ActixAdminViewModelSerializable, ActixAdminViewModelFieldType };
    pub use actix_admin_macros::{ DeriveActixAdmin, DeriveActixAdminModel, DeriveActixAdminViewModel, DeriveActixAdminEnumSelectList, DeriveActixAdminModelSelectList };
    pub use crate::{ ActixAdminAppDataTrait, ActixAdmin, ActixAdminConfiguration };
    pub use crate::{ hashmap, ActixAdminSelectListTrait };
    pub use crate::routes::{ create_or_edit_post, get_admin_ctx };
    pub use crate::{ TERA };
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
    async fn get_key_value(db: &DatabaseConnection) -> Vec<(String, String)>;
}


#[derive(Clone)]
pub struct ActixAdminConfiguration {
    pub enable_auth: bool,
    pub user_is_logged_in: Option<for<'a> fn(&'a Session) -> bool>,
    pub login_link: Option<String>,
    pub logout_link: Option<String>
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
