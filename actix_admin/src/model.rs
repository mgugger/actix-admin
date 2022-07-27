use crate::ActixAdminViewModelField;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use std::collections::HashMap;

#[async_trait]
pub trait ActixAdminModelTrait {
    async fn list_model(
        db: &DatabaseConnection,
        page: usize,
        posts_per_page: usize,
    ) -> (usize, Vec<ActixAdminModel>);
    fn get_fields() -> Vec<ActixAdminViewModelField>;
    fn validate_model(model: &ActixAdminModel) -> HashMap<String, String>;
    // function to be overridable for custom error handling
    fn validate(&self) -> HashMap<String, String> {
        return HashMap::new();
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminModel {
    pub primary_key: Option<String>,
    pub values: HashMap<String, String>,
    pub errors: HashMap<String, String>,
}

impl From<String> for ActixAdminModel {
    fn from(string: String) -> Self {
        let mut hashmap = HashMap::new();
        let key_values: Vec<&str> = string.split('&').collect();
        for key_value in key_values {
            if !key_value.is_empty() {
                let mut iter = key_value.splitn(2, '=');
                hashmap.insert(
                    iter.next().unwrap().to_string(),
                    iter.next().unwrap().to_string(),
                );
            }
        }

        ActixAdminModel {
            primary_key: None,
            values: hashmap,
            errors: HashMap::new(),
        }
    }
}

impl ActixAdminModel {
    pub fn get_value<T: std::str::FromStr>(&self, key: &str, is_option_or_string: bool) -> Result<Option<T>, String> {
        let value = self.values.get(key);

        let res: Result<Option<T>, String> = match value {
            Some(val) => {
                if val.is_empty() && is_option_or_string {
                    return Ok(None);
                }

                let parsed_val = val.parse::<T>();

                match parsed_val {
                    Ok(val) => Ok(Some(val)),
                    Err(_) => Err("Invalid Value".to_string()),
                }
            }
            _ => {
                match is_option_or_string {
                    true => Ok(None),
                    false => Err("Invalid Value".to_string()) // a missing value in the form for a non-optional value
                } 
            } 
        };

        res
    }

    pub fn has_errors(&self) -> bool {
        return &self.errors.len() != &0;
    }
}
