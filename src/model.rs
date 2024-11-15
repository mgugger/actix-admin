use crate::view_model::{ActixAdminViewModelFilter, ActixAdminViewModelParams};
use crate::{ActixAdminError, ActixAdminViewModelField};
use actix_multipart::{Multipart, MultipartError};
use actix_web::web::Bytes;
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime};
use futures_util::stream::StreamExt as _;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde_derive::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[async_trait]
pub trait ActixAdminModelTrait {
    async fn list_model(
        db: &DatabaseConnection,
        params: &ActixAdminViewModelParams,
        filter_values: HashMap<String, Option<String>>
    ) -> Result<(Option<u64>, Vec<ActixAdminModel>), ActixAdminError>;
    fn get_fields() -> &'static [ActixAdminViewModelField];
    fn validate_model(model: &mut ActixAdminModel);
    async fn load_foreign_keys(models: &mut Vec<ActixAdminModel>, db: &DatabaseConnection);
}

pub trait ActixAdminModelValidationTrait<T> {
    fn validate(_model: &T) -> HashMap<String, String> {
        return HashMap::new();
    }
}

pub struct ActixAdminModelFilter<E: EntityTrait> {
    pub name: String,
    pub filter_type: ActixAdminModelFilterType,
    pub filter: fn(sea_orm::Select<E>, Option<String>) -> sea_orm::Select<E>,
    pub values: Option<Vec<(String, String)>>
}

#[derive(Clone, Debug, Serialize)]
pub enum ActixAdminModelFilterType {
    Text,
    SelectList,
    Date,
    DateTime,
    Checkbox
}

#[async_trait]
pub trait ActixAdminModelFilterTrait<E: EntityTrait> {
    fn get_filter() -> Vec<ActixAdminModelFilter<E>> {
        Vec::new()
    }
    async fn get_filter_values(_filter: &ActixAdminModelFilter<E>, _db: &DatabaseConnection)-> Option<Vec<(String, String)>> {
        None
    }
}

impl<T: EntityTrait> From<ActixAdminModelFilter<T>> for ActixAdminViewModelFilter {
    fn from(filter: ActixAdminModelFilter<T>) -> Self {
        ActixAdminViewModelFilter {
            name: filter.name,
            value: None,
            values: None,
            filter_type: Some(filter.filter_type)
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminModel {
    pub primary_key: Option<String>,
    pub values: HashMap<String, String>,
    pub fk_values: HashMap<String, String>,
    pub errors: HashMap<String, String>,
    pub custom_errors: HashMap<String, String>,
    pub display_name: Option<String>,
}

impl ActixAdminModel {
    pub fn create_empty() -> ActixAdminModel {
        ActixAdminModel {
            primary_key: None,
            values: HashMap::new(),
            errors: HashMap::new(),
            custom_errors: HashMap::new(),
            fk_values: HashMap::new(),
            display_name: None
        }
    }

    pub async fn create_from_payload(
        id: Option<i32>,
        mut payload: Multipart, file_upload_folder: &str
    ) -> Result<ActixAdminModel, MultipartError> {
        let mut hashmap = HashMap::<String, String>::new();

        while let Some(item) = payload.next().await {
            let mut field = item?;

            let mut binary_data: Vec<Bytes> = Vec::new();
            while let Some(chunk) = field.next().await {
                binary_data.push(chunk.unwrap());
                //println!("-- CHUNK: \n{:?}", String::from_utf8(chunk.map_or(Vec::new(), |c| c.to_vec())));
                // let res_string = String::from_utf8(chunk.map_or(Vec::new(), |c| c.to_vec()));
            }
            let binary_data = binary_data.concat();
            if field.content_disposition().expect("expected content disposition").get_filename().is_some() {
                let mut filename = field
                    .content_disposition()
                    .expect("expected content disposition")
                    .get_filename()
                    .unwrap()
                    .to_string();

                let mut file_path = format!("{}/{}", file_upload_folder, filename);
                let file_exists = std::path::Path::new(&file_path).exists();
                // Avoid overwriting existing files
                if file_exists {
                    filename =  format!("{}_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(), filename);
                    file_path = format!("{}/{}", file_upload_folder, filename);
                }

                let file = File::create(file_path);
                let _res = file.unwrap().write_all(&binary_data);

                hashmap.insert(
                    field.name().expect("expected file name").to_string(),
                    filename.clone()
                );
            } else {
                let res_string = String::from_utf8(binary_data);
                if res_string.is_ok() {
                    hashmap.insert(field.name().expect("expected file name").to_string(), res_string.unwrap());
                }
            }
        }

        Ok(ActixAdminModel {
            primary_key: match id {
                Some(id) => Some(id.to_string()),
                None => None
            },
            values: hashmap,
            errors: HashMap::new(),
            custom_errors: HashMap::new(),
            fk_values: HashMap::new(),
            display_name: None
        })
    }

    pub fn get_value<T: std::str::FromStr>(
        &self,
        key: &str,
        is_option_or_string: bool,
        is_allowed_to_be_empty: bool
    ) -> Result<Option<T>, String> {
        self.get_value_by_closure(key, is_option_or_string, is_allowed_to_be_empty, |val| val.parse::<T>())
    }

    pub fn get_datetime(
        &self,
        key: &str,
        is_option_or_string: bool,
        is_allowed_to_be_empty: bool
    ) -> Result<Option<NaiveDateTime>, String> {
        self.get_value_by_closure(key, is_option_or_string, is_allowed_to_be_empty, |val| {
            NaiveDateTime::parse_from_str(val, "%Y-%m-%dT%H:%M")
        })
    }

    pub fn get_date(
        &self,
        key: &str,
        is_option_or_string: bool,
        is_allowed_to_be_empty: bool
    ) -> Result<Option<NaiveDate>, String> {
        self.get_value_by_closure(key, is_option_or_string, is_allowed_to_be_empty, |val| {
            NaiveDate::parse_from_str(val, "%Y-%m-%d")
        })
    }

    pub fn get_bool(&self, key: &str, is_option_or_string: bool, is_allowed_to_be_empty: bool) -> Result<Option<bool>, String> {
        let val = self.get_value_by_closure(key, is_option_or_string, is_allowed_to_be_empty ,|val| {
            if !val.is_empty() && (val == "true" || val == "yes") {
                Ok(true)
            } else {
                Ok(false)
            }
        });
        // not selected bool field equals to false and not to missing
        match val {
            Ok(val) => Ok(val),
            Err(_) => Ok(Some(false)),
        }
    }

    fn get_value_by_closure<T: std::str::FromStr>(
        &self,
        key: &str,
        is_option_or_string: bool,
        is_allowed_to_be_empty: bool,
        f: impl Fn(&String) -> Result<T, <T as std::str::FromStr>::Err>,
    ) -> Result<Option<T>, String> {
        let value = self.values.get(key);

        let res: Result<Option<T>, String> = match value {
            Some(val) => {
                match (val.is_empty(), is_option_or_string, is_allowed_to_be_empty) {
                    (true, true, true) => return Ok(None),
                    (true, true, false) => return Err("Cannot be empty".to_string()),
                    _ => {}
                };

                let parsed_val = f(val);

                match parsed_val {
                    Ok(val) => Ok(Some(val)),
                    Err(_) => Err("Invalid Value".to_string()),
                }
            }
            _ => {
                match (is_option_or_string, is_allowed_to_be_empty) {
                    (true, true) => Ok(None),
                    (true, false) => Err("Cannot be empty".to_string()),
                    (false, _) => Err("Invalid Value".to_string()), // a missing value in the form for a non-optional value
                }
            }
        };

        res
    }

    pub fn has_errors(&self) -> bool {
        return (&self.errors.len() + &self.custom_errors.len()) != 0 as usize;
    }
}
