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
    pub use crate::view_model::{ ActixAdminViewModel, ActixAdminViewModelTrait};
    pub use actix_admin_macros::{ DeriveActixAdminModel };
    pub use crate::{ ActixAdminAppDataTrait, ActixAdmin};
    pub use crate::{ hashmap };
}

use crate::prelude::*; 

#[macro_export]
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key.to_string(), $val.to_string()); )*
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

// ActixAdminModel

#[derive(Clone, Debug)]
pub struct ActixAdmin {
    pub entity_names: Vec<String>,
    pub view_models: HashMap<String, ActixAdminViewModel>,
}
