//! Unified request-side query parsing for list-like admin routes.
//!
//! `ListQuery` merges what used to be `Params`, `SearchParams`, ad-hoc
//! form-body field parsing (in `delete_many`) and `parse_filters_from_query`
//! into a single value that:
//!
//! * ignores unknown / unparseable keys (never panics on user input),
//! * normalises defaults against a [`ActixAdminViewModel`],
//! * knows how to round-trip itself back into a URL query string,
//! * can produce an [`ActixAdminViewModelParams`] for the ORM layer.
//!
//! Both the querystring (list / export / search) and the form body
//! (delete_many) go through the same parser so the URL-encoded shape of a
//! filter/pagination state is defined in exactly one place.

use serde_derive::Deserialize;

use crate::view_model::{
    ActixAdminFilterOperator, ActixAdminViewModelFilter, ActixAdminViewModelParams,
};
use crate::{ActixAdminViewModel, SortOrder};

use super::DEFAULT_ENTITIES_PER_PAGE;

/// The subset of a request's query string this crate understands. Extra
/// unknown keys are ignored.
#[derive(Debug, Deserialize, Default, Clone)]
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
/// (see [`ActixAdminFilterOperator`]'s `FromStr` impl). Operators are merged
/// onto the corresponding value filter; a bare `__op` without a matching
/// value is silently ignored.
pub fn parse_filters_from_query(qs: &str) -> Vec<ActixAdminViewModelFilter> {
    use std::collections::HashMap;

    let mut values: Vec<(String, Option<String>)> = Vec::new();
    let mut operators: HashMap<String, ActixAdminFilterOperator> = HashMap::new();

    for (key, value) in form_urlencoded::parse(qs.as_bytes()) {
        let Some(rest) = key.strip_prefix("filter_") else {
            continue;
        };
        if let Some(name) = rest.strip_suffix("__op") {
            if let Ok(op) = value.parse::<ActixAdminFilterOperator>() {
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

/// Fully-resolved list-page query state: pagination, search, sort and
/// filters, normalized against the entity's view model.
#[derive(Debug, Clone)]
pub struct ListQuery {
    pub page: u64,
    pub entities_per_page: u64,
    pub search: String,
    pub sort_by: String,
    pub sort_order: SortOrder,
    pub filters: Vec<ActixAdminViewModelFilter>,
}

impl ListQuery {
    /// Build a `ListQuery` from a raw querystring, filling in defaults
    /// from `view_model`. Never fails.
    pub fn from_query(qs: &str, view_model: &ActixAdminViewModel) -> Self {
        let params = Params::from_query(qs);
        let filters = parse_filters_from_query(qs);
        Self::from_params_and_filters(params, filters, view_model)
    }

    /// Build a `ListQuery` from a parsed `Params` plus a filter list.
    pub fn from_params_and_filters(
        params: Params,
        filters: Vec<ActixAdminViewModelFilter>,
        view_model: &ActixAdminViewModel,
    ) -> Self {
        ListQuery {
            page: params.page.unwrap_or(1),
            entities_per_page: params
                .entities_per_page
                .unwrap_or(DEFAULT_ENTITIES_PER_PAGE),
            search: params.search.unwrap_or_default(),
            sort_by: params
                .sort_by
                .unwrap_or_else(|| view_model.primary_key.clone()),
            sort_order: params.sort_order.unwrap_or(SortOrder::Asc),
            filters,
        }
    }

    /// Build a `ListQuery` from a form body of `(key, value)` pairs, used by
    /// `delete_many` where the pagination state travels in the POST body
    /// rather than the querystring.
    pub fn from_form(form: &[(String, String)], view_model: &ActixAdminViewModel) -> Self {
        let mut params = Params::default();
        for (k, v) in form {
            match k.as_str() {
                "page" => params.page = v.parse().ok(),
                "entities_per_page" => params.entities_per_page = v.parse().ok(),
                "search" => params.search = Some(v.clone()),
                "sort_by" => params.sort_by = Some(v.clone()),
                "sort_order" => params.sort_order = match v.as_str() {
                    "Asc" => Some(SortOrder::Asc),
                    "Desc" => Some(SortOrder::Desc),
                    _ => None,
                },
                _ => {}
            }
        }
        Self::from_params_and_filters(params, Vec::new(), view_model)
    }

    /// Serialize back into a URL querystring (without a leading `?`).
    /// Uses `serde_urlencoded` so encoding matches how we parse.
    pub fn to_query_string(&self) -> String {
        let pairs: Vec<(&str, String)> = vec![
            ("page", self.page.to_string()),
            ("entities_per_page", self.entities_per_page.to_string()),
            ("search", self.search.clone()),
            ("sort_by", self.sort_by.clone()),
            ("sort_order", self.sort_order.to_string()),
        ];
        serde_urlencoded::to_string(&pairs).unwrap_or_default()
    }

    /// Convert into the ORM-facing `ActixAdminViewModelParams`. `paginated`
    /// controls whether page/entities_per_page are forwarded; `export_csv`
    /// passes `false` to fetch all rows.
    pub fn to_view_model_params(
        &self,
        tenant_ref: Option<i32>,
        paginated: bool,
    ) -> ActixAdminViewModelParams {
        ActixAdminViewModelParams {
            page: if paginated { Some(self.page) } else { None },
            entities_per_page: if paginated {
                Some(self.entities_per_page)
            } else {
                None
            },
            viewmodel_filter: self.filters.clone(),
            search: self.search.clone(),
            sort_by: self.sort_by.clone(),
            sort_order: self.sort_order.clone(),
            tenant_ref,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_from_query_handles_garbage() {
        let p = Params::from_query("page=nope&entities_per_page=xyz&random=1");
        assert!(p.page.is_none());
        assert!(p.entities_per_page.is_none());
    }

    #[test]
    fn params_from_query_parses_valid_input() {
        let p = Params::from_query(
            "page=3&entities_per_page=20&search=foo&sort_by=name&sort_order=Desc",
        );
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
        assert!(filters[1].value.is_none());
    }

    #[test]
    fn filter_parser_survives_odd_percent_encoding() {
        let _ = parse_filters_from_query("filter_x=%ZZ");
        let _ = parse_filters_from_query("&&filter_=v&filter_a");
    }

    #[test]
    fn filter_parser_handles_equals_in_value() {
        let filters = parse_filters_from_query("filter_query=a=b");
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].value.as_deref(), Some("a=b"));
    }

    #[test]
    fn filter_parser_decodes_plus_as_space_in_key_and_value() {
        let filters = parse_filters_from_query("filter_Post+with+Tom+Select=hello+world");
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].name, "Post with Tom Select");
        assert_eq!(filters[0].value.as_deref(), Some("hello world"));
    }

    #[test]
    fn filter_parser_decodes_percent_encoded_key_and_value() {
        let filters = parse_filters_from_query("filter_Post%20with%20Tom%20Select=a%2Fb");
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].name, "Post with Tom Select");
        assert_eq!(filters[0].value.as_deref(), Some("a/b"));
    }
}
