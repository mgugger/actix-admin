use lazy_static::lazy_static;
use sea_orm::DatabaseConnection;
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
    static ref TERA: Tera = {
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
    pub login_link: String,
    pub logout_link: String
}

#[derive(Clone)]
pub struct ActixAdmin {
    pub entity_names: Vec<String>,
    pub view_models: HashMap<String, ActixAdminViewModel>,
    pub configuration: ActixAdminConfiguration
}
