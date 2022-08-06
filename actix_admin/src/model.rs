use crate::ActixAdminViewModelField;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use std::collections::HashMap;
use actix_multipart:: {Multipart, MultipartError} ;
use futures_util::stream::StreamExt as _;
use chrono::{NaiveDateTime, NaiveDate};
use sea_orm::prelude::*;

#[async_trait]
pub trait ActixAdminModelTrait {
    async fn list_model(
        db: &DatabaseConnection,
        page: usize,
        posts_per_page: usize,
        search: &String
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


impl ActixAdminModel {
    pub fn create_empty() -> ActixAdminModel {
        ActixAdminModel {
            primary_key: None,
            values: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    pub async fn create_from_payload(mut payload: Multipart) -> Result<ActixAdminModel, MultipartError> {
        let mut hashmap = HashMap::<String, String>::new();
    
        while let Some(item) = payload.next().await {
            let mut field = item?;
    
            // TODO: how to handle binary chunks?
            while let Some(chunk) = field.next().await {
                //println!("-- CHUNK: \n{:?}", String::from_utf8(chunk.map_or(Vec::new(), |c| c.to_vec())));
                let res_string = String::from_utf8(chunk.map_or(Vec::new(), |c| c.to_vec()));
                if res_string.is_ok() {
                    hashmap.insert(
                        field.name().to_string(),
                        res_string.unwrap()
                    );
                }
            }
        }

        Ok(ActixAdminModel {
            primary_key: None,
            values: hashmap,
            errors: HashMap::new(),
        })
    }

    pub fn get_value<T: std::str::FromStr>(&self, key: &str, is_option_or_string: bool) -> Result<Option<T>, String> {
        self.get_value_by_closure(key, is_option_or_string, |val| val.parse::<T>())
    }

    pub fn get_datetime(&self, key: &str, is_option_or_string: bool) -> Result<Option<DateTime>, String> {
        self.get_value_by_closure(key, is_option_or_string, |val| NaiveDateTime::parse_from_str(val, "%Y-%m-%dT%H:%M"))
    }

    pub fn get_date(&self, key: &str, is_option_or_string: bool) -> Result<Option<Date>, String> {
        self.get_value_by_closure(key, is_option_or_string, |val| NaiveDate::parse_from_str(val, "%Y-%m-%d"))
    }

    pub fn get_bool(&self, key: &str, is_option_or_string: bool) -> Result<Option<bool>, String> {
        let val = self.get_value_by_closure(key, is_option_or_string, |val| if !val.is_empty() { Ok(true) } else { Ok(false) });
        // not selected bool field equals to false and not to missing
        match val {
            Ok(val) => Ok(val),
            Err(_) => Ok(Some(false))
        }
    }

    fn get_value_by_closure<T: std::str::FromStr>(&self, key: &str, is_option_or_string: bool, f: impl Fn(&String) -> Result<T, <T as std::str::FromStr>::Err>) -> Result<Option<T>, String> {
        let value = self.values.get(key);

        let res: Result<Option<T>, String> = match value {
            Some(val) => {
                if val.is_empty() && is_option_or_string {
                    return Ok(None);
                }

                let parsed_val = f(val);

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
