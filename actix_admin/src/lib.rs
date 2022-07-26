use lazy_static::lazy_static;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use tera::{Tera};
use serde::{Serialize};

pub mod view_model;
pub mod model;
pub mod routes;
pub mod builder;

pub mod prelude {
    pub use crate::builder::{ ActixAdminBuilder, ActixAdminBuilderTrait};
    pub use crate::model::{ ActixAdminModel, ActixAdminModelTrait};
    pub use crate::view_model::{ ActixAdminViewModel, ActixAdminViewModelTrait, ActixAdminViewModelField};
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
    static ref TERA: Tera =
        Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
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
