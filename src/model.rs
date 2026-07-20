use crate::view_model::{
    ActixAdminFilterOperator, ActixAdminViewModelFilter, ActixAdminViewModelParams,
};
use crate::{ActixAdminError, ActixAdminErrorType, ActixAdminViewModelField};
use actix_multipart::Multipart;
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime};
use futures_util::stream::StreamExt as _;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde_derive::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum total upload size (default 25MB). Individual deployments should
/// enforce their own limits via `actix_multipart::form::MultipartFormConfig`
/// but this provides a defensive per-field cap so that a single field cannot
/// exhaust memory in `create_from_payload`.
pub const DEFAULT_MAX_FIELD_SIZE_BYTES: usize = 25 * 1024 * 1024;

/// Sanitize a user-provided filename so that it can never traverse outside the
/// upload directory. Strips path separators, `..`, control chars and NULs, and
/// falls back to a timestamp-based name if the result would be empty.
pub fn sanitize_upload_filename(raw: &str) -> String {
    // Take just the last path component. Split manually on both '/' and '\\'
    // so we correctly reject Windows-style traversal even on unix hosts.
    let last = raw.rsplit(['/', '\\']).next().unwrap_or("");

    let cleaned: String = sanitize_filename::sanitize_with_options(
        last,
        sanitize_filename::Options {
            windows: true,
            truncate: true,
            replacement: "_",
        },
    )
    .chars()
    .filter(|c| !c.is_control() && *c != '\0')
    .collect();
    let cleaned = cleaned.trim_matches(|c: char| c == '.' || c.is_whitespace());
    if cleaned.is_empty() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        format!("upload_{now}")
    } else {
        cleaned.to_string()
    }
}

#[async_trait]
pub trait ActixAdminModelTrait {
    async fn list_model(
        db: &DatabaseConnection,
        params: &ActixAdminViewModelParams,
        filter_values: HashMap<String, Option<String>>,
    ) -> Result<(Option<u64>, Vec<ActixAdminModel>), ActixAdminError>;
    fn get_fields() -> &'static [ActixAdminViewModelField];
    fn validate_model(model: &mut ActixAdminModel);
    async fn load_foreign_keys(models: &mut [ActixAdminModel], db: &DatabaseConnection);
}

pub trait ActixAdminModelValidationTrait<T> {
    fn validate(_model: &T) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// A single filter registered on an entity via `ActixAdminModelFilterTrait`.
///
/// The `filter` closure receives the current query and the user-provided
/// value (a plain `Option<String>`). If you also want to react to the user's
/// choice of operator ("contains", ">", "<", ...), populate `operators` with
/// the operators you support and read `operator` off the request via the
/// `filter_with_op` closure alternative.
pub struct ActixAdminModelFilter<E: EntityTrait> {
    pub name: String,
    pub filter_type: ActixAdminModelFilterType,
    /// Legacy value-only filter. Called when `filter_with_op` is `None`.
    pub filter: fn(sea_orm::Select<E>, Option<String>) -> sea_orm::Select<E>,
    pub values: Option<Vec<(String, String)>>,
    pub foreign_key: Option<String>,
    /// Operators the user may pick from. When empty, no operator selector is
    /// rendered.
    pub operators: Vec<ActixAdminFilterOperator>,
    /// Operator-aware alternative to `filter`. When set, it is called instead
    /// of `filter`.
    #[allow(clippy::type_complexity)]
    pub filter_with_op: Option<
        fn(
            sea_orm::Select<E>,
            Option<String>,
            Option<ActixAdminFilterOperator>,
        ) -> sea_orm::Select<E>,
    >,
}

#[derive(Clone, Debug, Serialize)]
pub enum ActixAdminModelFilterType {
    Text,
    SelectList,
    Date,
    DateTime,
    Checkbox,
    TomSelectSearch,
}

impl<E: EntityTrait> ActixAdminModelFilter<E> {
    /// Build a minimal legacy value-only filter. Equivalent to setting
    /// `operators = vec![]` and `filter_with_op = None`.
    pub fn new(
        name: impl Into<String>,
        filter_type: ActixAdminModelFilterType,
        filter: fn(sea_orm::Select<E>, Option<String>) -> sea_orm::Select<E>,
    ) -> Self {
        Self {
            name: name.into(),
            filter_type,
            filter,
            values: None,
            foreign_key: None,
            operators: Vec::new(),
            filter_with_op: None,
        }
    }

    pub fn with_operators(mut self, operators: Vec<ActixAdminFilterOperator>) -> Self {
        self.operators = operators;
        self
    }

    pub fn with_operator_filter(
        mut self,
        f: fn(
            sea_orm::Select<E>,
            Option<String>,
            Option<ActixAdminFilterOperator>,
        ) -> sea_orm::Select<E>,
    ) -> Self {
        self.filter_with_op = Some(f);
        self
    }

    pub fn with_foreign_key(mut self, fk: impl Into<String>) -> Self {
        self.foreign_key = Some(fk.into());
        self
    }

    pub fn with_values(mut self, values: Vec<(String, String)>) -> Self {
        self.values = Some(values);
        self
    }
}

#[async_trait]
pub trait ActixAdminModelFilterTrait<E: EntityTrait> {
    fn get_filter() -> Vec<ActixAdminModelFilter<E>> {
        Vec::new()
    }
    async fn get_filter_values(
        _filter: &ActixAdminModelFilter<E>,
        _db: &DatabaseConnection,
    ) -> Option<Vec<(String, String)>> {
        None
    }
}

impl<T: EntityTrait> From<ActixAdminModelFilter<T>> for ActixAdminViewModelFilter {
    fn from(filter: ActixAdminModelFilter<T>) -> Self {
        ActixAdminViewModelFilter {
            name: filter.name,
            value: None,
            values: None,
            filter_type: Some(filter.filter_type),
            foreign_key: None,
            operators: filter.operators,
            operator: None,
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
            display_name: None,
        }
    }

    pub async fn create_from_payload(
        id: Option<String>,
        mut payload: Multipart,
        file_upload_folder: &str,
    ) -> Result<ActixAdminModel, ActixAdminError> {
        let mut hashmap = HashMap::<String, String>::new();

        while let Some(item) = payload.next().await {
            let mut field = item?;

            let mut binary_data: Vec<u8> = Vec::new();
            while let Some(chunk) = field.next().await {
                let chunk = chunk?;
                if binary_data.len().saturating_add(chunk.len()) > DEFAULT_MAX_FIELD_SIZE_BYTES {
                    return Err(ActixAdminError::new(
                        ActixAdminErrorType::UploadError,
                        "Uploaded field exceeds maximum size",
                    ));
                }
                binary_data.extend_from_slice(&chunk);
            }

            let content_disposition = match field.content_disposition() {
                Some(cd) => cd.clone(),
                None => continue,
            };
            let field_name = match content_disposition.get_name() {
                Some(name) => name.to_string(),
                None => continue,
            };

            if let Some(raw_filename) = content_disposition.get_filename() {
                // Skip empty file uploads silently (browsers submit empty file fields).
                if raw_filename.is_empty() && binary_data.is_empty() {
                    continue;
                }

                let mut filename = sanitize_upload_filename(raw_filename);

                let base = PathBuf::from(file_upload_folder);
                let mut file_path = base.join(&filename);

                // Avoid overwriting existing files by prefixing a timestamp.
                if file_path.exists() {
                    let ts = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0);
                    filename = format!("{ts}_{filename}");
                    file_path = base.join(&filename);
                }

                // Defense in depth: reject any joined path that escapes the base.
                let canonical_base = base.canonicalize().unwrap_or_else(|_| base.clone());
                let parent = file_path.parent().unwrap_or(&base);
                let canonical_parent = parent
                    .canonicalize()
                    .unwrap_or_else(|_| parent.to_path_buf());
                if !canonical_parent.starts_with(&canonical_base) {
                    return Err(ActixAdminError::new(
                        ActixAdminErrorType::UploadError,
                        "Uploaded filename resolves outside the upload directory",
                    ));
                }

                let mut f = File::create(&file_path)?;
                f.write_all(&binary_data)?;

                hashmap.insert(field_name, filename);
            } else if let Ok(res_string) = String::from_utf8(binary_data) {
                hashmap.insert(field_name, res_string);
            }
        }

        Ok(ActixAdminModel {
            primary_key: id,
            values: hashmap,
            ..ActixAdminModel::create_empty()
        })
    }

    pub fn get_value<T: std::str::FromStr>(
        &self,
        key: &str,
        is_option_or_string: bool,
        is_allowed_to_be_empty: bool,
    ) -> Result<Option<T>, String> {
        self.get_value_by_closure(key, is_option_or_string, is_allowed_to_be_empty, |val| {
            val.parse::<T>()
        })
    }

    pub fn get_datetime(
        &self,
        key: &str,
        is_option_or_string: bool,
        is_allowed_to_be_empty: bool,
    ) -> Result<Option<NaiveDateTime>, String> {
        self.get_value_by_closure(key, is_option_or_string, is_allowed_to_be_empty, |val| {
            NaiveDateTime::parse_from_str(val, "%Y-%m-%dT%H:%M")
        })
    }

    pub fn get_date(
        &self,
        key: &str,
        is_option_or_string: bool,
        is_allowed_to_be_empty: bool,
    ) -> Result<Option<NaiveDate>, String> {
        self.get_value_by_closure(key, is_option_or_string, is_allowed_to_be_empty, |val| {
            NaiveDate::parse_from_str(val, "%Y-%m-%d")
        })
    }

    pub fn get_bool(
        &self,
        key: &str,
        is_option_or_string: bool,
        is_allowed_to_be_empty: bool,
    ) -> Result<Option<bool>, String> {
        // A missing/invalid bool from a checkbox means "unchecked".
        let val =
            self.get_value_by_closure(key, is_option_or_string, is_allowed_to_be_empty, |val| {
                Ok::<bool, std::str::ParseBoolError>(matches!(val.as_str(), "true" | "yes"))
            });
        Ok(val.unwrap_or(Some(false)))
    }

    fn get_value_by_closure<T: std::str::FromStr>(
        &self,
        key: &str,
        is_option_or_string: bool,
        is_allowed_to_be_empty: bool,
        f: impl Fn(&String) -> Result<T, <T as std::str::FromStr>::Err>,
    ) -> Result<Option<T>, String> {
        match self.values.get(key) {
            Some(val) => {
                if val.is_empty() && is_option_or_string {
                    return if is_allowed_to_be_empty {
                        Ok(None)
                    } else {
                        Err("Cannot be empty".to_string())
                    };
                }
                f(val).map(Some).map_err(|_| "Invalid Value".to_string())
            }
            None => match (is_option_or_string, is_allowed_to_be_empty) {
                (true, true) => Ok(None),
                (true, false) => Err("Cannot be empty".to_string()),
                (false, _) => Err("Invalid Value".to_string()),
            },
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty() || !self.custom_errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model_with(key: &str, value: &str) -> ActixAdminModel {
        let mut m = ActixAdminModel::create_empty();
        m.values.insert(key.to_string(), value.to_string());
        m
    }

    // ---- sanitize_upload_filename ----

    #[test]
    fn sanitize_strips_traversal() {
        assert_eq!(sanitize_upload_filename("../../etc/passwd"), "passwd");
        assert_eq!(sanitize_upload_filename("..\\..\\evil.exe"), "evil.exe");
        assert_eq!(sanitize_upload_filename("/absolute/path/x.txt"), "x.txt");
    }

    #[test]
    fn sanitize_removes_control_chars_and_nul() {
        let out = sanitize_upload_filename("foo\0bar\nbaz.txt");
        assert!(!out.contains('\0'));
        assert!(!out.contains('\n'));
        assert!(out.ends_with(".txt"));
    }

    #[test]
    fn sanitize_empty_or_dotfile_falls_back() {
        let out = sanitize_upload_filename("");
        assert!(out.starts_with("upload_"));
        let out = sanitize_upload_filename("...");
        // Result must not resolve to a traversal or hidden dotfile.
        assert!(!out.contains(".."));
        assert!(!out.starts_with('.'));
        assert!(!out.is_empty());
    }

    // ---- get_value matrix ----

    #[test]
    fn get_value_present_parses() {
        let m = model_with("n", "42");
        let v: Option<i32> = m.get_value("n", false, false).unwrap();
        assert_eq!(v, Some(42));
    }

    #[test]
    fn get_value_invalid_returns_err() {
        let m = model_with("n", "abc");
        let r: Result<Option<i32>, _> = m.get_value("n", false, false);
        assert!(r.is_err());
    }

    #[test]
    fn get_value_empty_string_option_allowed_returns_none() {
        let m = model_with("s", "");
        let r: Result<Option<String>, _> = m.get_value("s", true, true);
        assert_eq!(r.unwrap(), None);
    }

    #[test]
    fn get_value_empty_string_option_not_allowed_returns_err() {
        let m = model_with("s", "");
        let r: Result<Option<String>, _> = m.get_value("s", true, false);
        assert!(r.is_err());
    }

    #[test]
    fn get_value_missing_option_allowed_returns_none() {
        let m = ActixAdminModel::create_empty();
        let r: Result<Option<String>, _> = m.get_value("missing", true, true);
        assert_eq!(r.unwrap(), None);
    }

    #[test]
    fn get_value_missing_option_not_allowed_returns_err() {
        let m = ActixAdminModel::create_empty();
        let r: Result<Option<String>, _> = m.get_value("missing", true, false);
        assert!(r.is_err());
    }

    #[test]
    fn get_value_missing_non_option_returns_err() {
        let m = ActixAdminModel::create_empty();
        let r: Result<Option<i32>, _> = m.get_value("missing", false, true);
        assert!(r.is_err());
    }

    // ---- get_bool ----

    #[test]
    fn get_bool_true_yes() {
        let m = model_with("b", "true");
        assert_eq!(m.get_bool("b", false, true).unwrap(), Some(true));
        let m = model_with("b", "yes");
        assert_eq!(m.get_bool("b", false, true).unwrap(), Some(true));
    }

    #[test]
    fn get_bool_other_values_are_false() {
        let m = model_with("b", "off");
        assert_eq!(m.get_bool("b", false, true).unwrap(), Some(false));
    }

    #[test]
    fn get_bool_missing_falls_back_to_false() {
        let m = ActixAdminModel::create_empty();
        assert_eq!(m.get_bool("b", false, false).unwrap(), Some(false));
    }

    // ---- get_date / get_datetime ----

    #[test]
    fn get_date_valid() {
        let m = model_with("d", "2024-01-02");
        let d = m.get_date("d", false, false).unwrap().unwrap();
        assert_eq!(d, chrono::NaiveDate::from_ymd_opt(2024, 1, 2).unwrap());
    }

    #[test]
    fn get_date_invalid() {
        let m = model_with("d", "nope");
        assert!(m.get_date("d", false, false).is_err());
    }

    #[test]
    fn get_datetime_valid_local_form() {
        let m = model_with("d", "2024-01-02T03:04");
        let dt = m.get_datetime("d", false, false).unwrap().unwrap();
        assert_eq!(
            dt,
            chrono::NaiveDate::from_ymd_opt(2024, 1, 2)
                .unwrap()
                .and_hms_opt(3, 4, 0)
                .unwrap()
        );
    }

    // ---- has_errors ----

    #[test]
    fn has_errors_reports_both_maps() {
        let mut m = ActixAdminModel::create_empty();
        assert!(!m.has_errors());
        m.errors.insert("a".into(), "b".into());
        assert!(m.has_errors());
        m.errors.clear();
        m.custom_errors.insert("a".into(), "b".into());
        assert!(m.has_errors());
    }
}
