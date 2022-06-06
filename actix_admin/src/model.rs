use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use serde::{Serialize};
use std::collections::HashMap;

use crate::ActixAdminField;

#[async_trait]
pub trait ActixAdminModelTrait {
    async fn list_model(
        db: &DatabaseConnection,
        page: usize,
        posts_per_page: usize,
    ) -> (usize, Vec<ActixAdminModel>);
    fn get_fields() -> Vec<(String, ActixAdminField)>;
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminModel {
    pub primary_key: Option<String>,
    pub values: HashMap<String, String>,
}

impl From<String> for ActixAdminModel {
    fn from(string: String) -> Self {
        let mut hashmap = HashMap::new();
        let key_values: Vec<&str> = string.split('&').collect();
        for key_value in key_values {
            let mut iter = key_value.splitn(2, '=');
            hashmap.insert(
                iter.next().unwrap().to_string(),
                iter.next().unwrap().to_string(),
            );
        }

        ActixAdminModel { primary_key: None, values: hashmap }
    }
}

impl ActixAdminModel {
    pub fn get_value<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        let value = self.values.get(key).unwrap().to_string().parse::<T>();
        match value {
            Ok(val) => Some(val),
            Err(_) => None, //panic!("key {} could not be parsed", key)
        }
    }
}