use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use serde::{Serialize};
use std::collections::HashMap;
use crate::ActixAdminViewModelField;
use crate::ActixAdminError;

#[async_trait]
pub trait ActixAdminModelTrait {
    async fn list_model(
        db: &DatabaseConnection,
        page: usize,
        posts_per_page: usize,
    ) -> (usize, Vec<ActixAdminModel>);
    fn get_fields() -> Vec<ActixAdminViewModelField>;
    fn validate_model(model: &ActixAdminModel) -> Vec<ActixAdminError>;
    
    // function to be overridable for custom error handling
    fn validate(&self) -> Vec<ActixAdminError> {
        return Vec::new()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminModel {
    pub primary_key: Option<String>,
    pub values: HashMap<String, String>,
    pub errors: Vec<ActixAdminError>,
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

        ActixAdminModel { primary_key: None, values: hashmap, errors: Vec::new() }
    }
}

impl ActixAdminModel {
    pub fn get_value<T: std::str::FromStr>(&self, key: &str) -> Result<Option<T>, ActixAdminError> {
        let value = self.values.get(key);
        let res: Result<Option<T>, ActixAdminError> = match value {
            Some(val) => {
                let parsed_val = val.parse::<T>();   
                match parsed_val {
                    Ok(val) => Ok(Some(val)),
                    Err(_) => Err(ActixAdminError { 
                        field_name: Some(key.to_string()), 
                        error: "Invalid Value".to_string()
                    })
                }
            },
            _ => Ok(None)
        };

        res
    }

    pub fn has_errors(&self) -> bool {
        return &self.errors.len() != &0
    }
}