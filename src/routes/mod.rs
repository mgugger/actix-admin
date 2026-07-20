mod create_or_edit_get;
pub use create_or_edit_get::{create_get, edit_get};

mod create_or_edit_post;
pub use create_or_edit_post::{ create_post, edit_post, create_or_edit_post };

mod index;
pub use index::{ index, not_found, get_admin_ctx };

mod list;
pub use list::{ list, SortOrder, export_csv };

mod show;
pub use show::show;

mod delete;
pub use delete::{ delete, delete_many };

mod bulk_action;
pub use bulk_action::{ bulk_action, ActixAdminBulkActionDispatch };

mod helpers;
pub use helpers::{add_auth_context, forbid_if_denied, render_template, render_unauthorized, user_can_access_page, user_can_perform, validate_sort_by, view_model_or_500, AdminAction};

mod file;
pub use file::{download, delete_file};

mod card_grid;
pub use card_grid::display_card_grid;

mod search;
pub use search::search;

use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Params {
    pub(crate) page: Option<u64>,
    pub(crate) entities_per_page: Option<u64>,
    pub(crate) search: Option<String>,
    pub(crate) sort_by: Option<String>,
    pub(crate) sort_order: Option<SortOrder>,
}

impl Params {
    /// Parse `Params` from a raw querystring, ignoring unknown keys. Never fails.
    pub fn from_query(qs: &str) -> Self {
        serde_urlencoded::from_str::<Self>(qs).unwrap_or_default()
    }
}

/// Parse `filter_<name>=value` querystring fragments into filters, without
/// panicking on malformed input.
///
/// Also recognises operator selectors of the form `filter_<name>__op=<op>`
/// (see [`crate::view_model::ActixAdminFilterOperator::from_str`]). Operators
/// are merged onto the corresponding value filter; a bare `__op` without a
/// matching value is silently ignored.
///
/// Uses `form_urlencoded` so that `+` is decoded as a space (matching the
/// way browsers submit HTML forms) and each key/value is decoded
/// independently.
pub(crate) fn parse_filters_from_query(qs: &str) -> Vec<crate::view_model::ActixAdminViewModelFilter> {
    use crate::view_model::{ActixAdminFilterOperator, ActixAdminViewModelFilter};
    use std::collections::HashMap;

    let mut values: Vec<(String, Option<String>)> = Vec::new();
    let mut operators: HashMap<String, ActixAdminFilterOperator> = HashMap::new();

    for (key, value) in form_urlencoded::parse(qs.as_bytes()) {
        let Some(rest) = key.strip_prefix("filter_") else { continue };
        if let Some(name) = rest.strip_suffix("__op") {
            if let Some(op) = ActixAdminFilterOperator::from_str(&value) {
                operators.insert(name.to_string(), op);
            }
            continue;
        }
        let v = if value.is_empty() {
            None
        } else {
            Some(value.into_owned())
        };
        values.push((rest.to_string(), v));
    }

    values
        .into_iter()
        .map(|(name, value)| {
            let operator = operators.remove(&name);
            ActixAdminViewModelFilter {
                name,
                value,
                values: None,
                filter_type: None,
                foreign_key: None,
                operators: Vec::new(),
                operator,
            }
        })
        .collect()
}

const DEFAULT_ENTITIES_PER_PAGE: u64 = 10;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_from_query_handles_garbage() {
        // Unknown keys and bad values must not panic.
        let p = Params::from_query("page=nope&entities_per_page=xyz&random=1");
        assert!(p.page.is_none());
        assert!(p.entities_per_page.is_none());
    }

    #[test]
    fn params_from_query_parses_valid_input() {
        let p = Params::from_query("page=3&entities_per_page=20&search=foo&sort_by=name&sort_order=Desc");
        assert_eq!(p.page, Some(3));
        assert_eq!(p.entities_per_page, Some(20));
        assert_eq!(p.search.as_deref(), Some("foo"));
        assert_eq!(p.sort_by.as_deref(), Some("name"));
        assert!(matches!(p.sort_order, Some(SortOrder::Desc)));
    }

    #[test]
    fn filter_parser_extracts_filter_prefixed_pairs() {
        let filters = parse_filters_from_query("page=1&filter_status=active&filter_owner=&other=x");
        let names: Vec<_> = filters.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["status", "owner"]);
        assert_eq!(filters[0].value.as_deref(), Some("active"));
        assert!(filters[1].value.is_none(), "empty value must be normalized to None");
    }

    #[test]
    fn filter_parser_survives_odd_percent_encoding() {
        // The parser must not panic on odd input, whatever the decoder chooses to do with it.
        let _ = parse_filters_from_query("filter_x=%ZZ");
        let _ = parse_filters_from_query("&&filter_=v&filter_a");
    }

    #[test]
    fn filter_parser_handles_equals_in_value() {
        // Values may contain '='.
        let filters = parse_filters_from_query("filter_query=a=b");
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].value.as_deref(), Some("a=b"));
    }

    #[test]
    fn filter_parser_decodes_plus_as_space_in_key_and_value() {
        // HTML forms submit spaces as `+`. Filter names with spaces (like
        // `Post with Tom Select`) must be decoded correctly, otherwise
        // they never match the registered filter.
        let filters =
            parse_filters_from_query("filter_Post+with+Tom+Select=hello+world");
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].name, "Post with Tom Select");
        assert_eq!(filters[0].value.as_deref(), Some("hello world"));
    }

    #[test]
    fn filter_parser_decodes_percent_encoded_key_and_value() {
        let filters = parse_filters_from_query(
            "filter_Post%20with%20Tom%20Select=a%2Fb",
        );
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].name, "Post with Tom Select");
        assert_eq!(filters[0].value.as_deref(), Some("a/b"));
    }
}
