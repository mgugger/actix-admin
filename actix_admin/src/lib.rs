use lazy_static::lazy_static;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use tera::{Tera, Result, Value, to_value, try_get_value };
use std::{ hash::BuildHasher};

pub mod view_model;
pub mod model;
pub mod routes;
pub mod builder;

pub mod prelude {
    pub use crate::builder::{ ActixAdminBuilder, ActixAdminBuilderTrait};
    pub use crate::model::{ ActixAdminModel, ActixAdminModelTrait};
    pub use crate::view_model::{ ActixAdminViewModel, ActixAdminViewModelTrait, ActixAdminViewModelField, ActixAdminViewModelFieldType };
    pub use actix_admin_macros::{ DeriveActixAdminModel, DeriveActixAdminSelectList };
    pub use crate::{ ActixAdminAppDataTrait, ActixAdmin };
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

pub fn get_html_input_type<S: BuildHasher>(value: &tera::Value, _: &HashMap<String, tera::Value, S>) -> Result<tera::Value> {
    let field = try_get_value!("get_html_input_type", "value", ActixAdminViewModelField, value);

    // TODO: convert to option
    if field.html_input_type != "" {
        return Ok(to_value(field.html_input_type).unwrap())
    }

    let html_input_type = match field.field_type {
        ActixAdminViewModelFieldType::Text => "text",
        ActixAdminViewModelFieldType::DateTime => "datetime-local",
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
pub trait ActixAdminSelectListTrait {
    fn get_key_value() -> Vec<(String, String)>;
}

// ActixAdminModel
#[derive(Clone, Debug)]
pub struct ActixAdmin {
    pub entity_names: Vec<String>,
    pub view_models: HashMap<String, ActixAdminViewModel>,
}
