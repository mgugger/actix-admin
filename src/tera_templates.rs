use serde::Deserialize;
use tera::{Kwargs, State, Tera, TeraResult, Value};

use crate::view_model::{ActixAdminViewModelField, ActixAdminViewModelFieldType};

/// Deserialize a tera `Value` into `T`, wrapping the error into a tera error
/// prefixed with the filter name (mimics the old `try_get_value!` macro).
fn from_value<'de, T: Deserialize<'de>>(filter_name: &str, value: &'de Value) -> TeraResult<T> {
    T::deserialize(value).map_err(|e| {
        tera::Error::message(format!(
            "Filter `{filter_name}` was called on an incorrect value: {e}"
        ))
    })
}

pub fn get_tera() -> Tera {
    // Register filters BEFORE adding templates: tera 2 validates filter
    // references when templates are added.
    let mut tera = Tera::new();
    tera.register_filter("get_html_input_type", get_html_input_type);
    tera.register_filter("get_html_input_class", get_html_input_class);
    tera.register_filter("get_icon", get_icon);
    tera.register_filter("get_regex_val", get_regex_val);
    tera.register_filter("shorten", shorten_filter);
    // Filters that existed in tera 1 but were removed in tera 2. We
    // provide compatibility shims so the shipped templates keep working.
    tera.register_filter("date", date_filter);
    tera.register_filter("filter", filter_attribute);

    load_templates_into(&mut tera);
    tera
}

fn shorten_filter(value: &Value, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
    let max_length: Option<u64> = kwargs.get("max_length")?;
    let input = value.as_str().unwrap_or("");

    if let Some(max) = max_length {
        if input.len() <= max as usize {
            Ok(Value::from(input))
        } else {
            let shortened: String = input.chars().take(max as usize).collect();
            Ok(Value::from(shortened))
        }
    } else {
        Ok(Value::from(input))
    }
}

fn get_html_input_class(value: &Value, _: Kwargs, _: &State) -> TeraResult<Value> {
    let field: ActixAdminViewModelField = from_value("get_html_input_class", value)?;
    let html_input_type = match field.field_type {
        ActixAdminViewModelFieldType::TextArea => "textarea",
        ActixAdminViewModelFieldType::Checkbox => "checkbox",
        _ => "input",
    };

    Ok(Value::from(html_input_type))
}

fn get_icon(value: &Value, _: Kwargs, _: &State) -> TeraResult<Value> {
    let field: String = from_value("get_icon", value)?;
    let font_awesome_icon = match field.as_str() {
        "true" => "<i class=\"fa-solid fa-check\"></i>",
        "false" => "<i class=\"fa-solid fa-xmark\"></i>",
        other => {
            return Err(tera::Error::message(format!(
                "get_icon: unsupported value '{other}' (expected 'true' or 'false')"
            )))
        }
    };

    Ok(Value::from(font_awesome_icon))
}

fn get_regex_val(value: &Value, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
    let field: ActixAdminViewModelField = from_value("get_regex_val", value)?;

    // `values` is expected to be a map keyed by field name.
    let values: &tera::Map = kwargs.must_get("values")?;
    let field_val = values.get(&tera::value::Key::Str(&field.field_name));

    match (field_val, field.list_regex_mask) {
        (Some(val), Some(r)) => {
            let val_str = val.to_string();
            let result_str = r.replace_all(&val_str, "*").into_owned();
            Ok(Value::from(result_str))
        }
        (Some(val), None) => Ok(val.clone()),
        (None, _) => Err(tera::Error::message(format!(
            "key '{}' not found in model values",
            field.field_name
        ))),
    }
}

fn get_html_input_type(value: &Value, _: Kwargs, _: &State) -> TeraResult<Value> {
    let field: ActixAdminViewModelField = from_value("get_html_input_type", value)?;

    // TODO: convert to option
    if !field.html_input_type.is_empty() {
        return Ok(Value::from(field.html_input_type));
    }

    let html_input_type = match field.field_type {
        ActixAdminViewModelFieldType::Text => "text",
        ActixAdminViewModelFieldType::DateTime => "datetime-local",
        ActixAdminViewModelFieldType::Date => "date",
        ActixAdminViewModelFieldType::Checkbox => "checkbox",
        ActixAdminViewModelFieldType::FileUpload => "file",
        _ => "text",
    };

    Ok(Value::from(html_input_type))
}

/// Minimal reimplementation of tera 1's `date` filter for the (limited) usage
/// in shipped templates: `{{ value | date(format="...") }}`.
///
/// Accepts either a stringly-typed RFC3339-ish datetime or a naive date; falls
/// back to returning the input unchanged if it cannot be parsed.
fn date_filter(value: &Value, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
    use chrono::{DateTime, NaiveDate, NaiveDateTime};

    let format: String = kwargs
        .get("format")?
        .unwrap_or_else(|| "%Y-%m-%d".to_string());
    let Some(input) = value.as_str() else {
        return Ok(value.clone());
    };

    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return Ok(Value::from(dt.format(&format).to_string()));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S") {
        return Ok(Value::from(dt.format(&format).to_string()));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S") {
        return Ok(Value::from(dt.format(&format).to_string()));
    }
    if let Ok(d) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        return Ok(Value::from(d.format(&format).to_string()));
    }
    Ok(Value::from(input))
}

/// Reimplementation of tera 1's `filter` filter:
/// `{{ array | filter(attribute="foo", value=<v>) }}` keeps only the elements
/// of `array` whose `foo` attribute equals `<v>` (or, if `value` is omitted,
/// only the elements where `foo` is defined and truthy).
fn filter_attribute(value: &Value, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
    let attribute: String = kwargs.must_get("attribute")?;
    let target: Option<&Value> = kwargs.get("value")?;

    let arr = value
        .as_array()
        .ok_or_else(|| tera::Error::message("`filter` expects an array as input".to_string()))?;

    let key = tera::value::Key::Str(&attribute);
    let filtered: Vec<Value> = arr
        .iter()
        .filter(|item| {
            let Some(map) = item.as_map() else {
                return false;
            };
            match (map.get(&key), target) {
                (Some(v), Some(t)) => v == t,
                (Some(v), None) => !v.is_none() && !v.is_undefined(),
                _ => false,
            }
        })
        .cloned()
        .collect();
    Ok(Value::from(filtered))
}

fn add_templates_to_tera(tera: &mut Tera, templates: &[(&'static str, &'static str)]) {
    tera.add_raw_templates(templates.iter().copied())
        .expect("failed to register built-in actix-admin templates");
}

#[cfg(all(feature = "bootstrapv5_css", feature = "bulma_css"))]
compile_error!(
    "features `bulma_css` and `bootstrapv5_css` cannot be enabled at the same time. \
     Disable default features with `default-features = false`."
);

#[cfg(not(any(feature = "bulma_css", feature = "bootstrapv5_css")))]
compile_error!("At least one CSS theme feature must be enabled: `bulma_css` or `bootstrapv5_css`.");

// Cargo Features for CSS
#[cfg(feature = "bulma_css")]
fn load_templates_into(tera: &mut Tera) {
    const TEMPLATES: &[(&str, &str)] = &[
        ("base.html", include_str!("templates/bulma/base.html")),
        ("list.html", include_str!("templates/bulma/list.html")),
        (
            "create_or_edit.html",
            include_str!("templates/bulma/create_or_edit.html"),
        ),
        ("head.html", include_str!("templates/bulma/head.html")),
        ("index.html", include_str!("templates/bulma/index.html")),
        ("loader.html", include_str!("templates/bulma/loader.html")),
        ("navbar.html", include_str!("templates/bulma/navbar.html")),
        (
            "not_found.html",
            include_str!("templates/bulma/not_found.html"),
        ),
        ("show.html", include_str!("templates/bulma/show.html")),
        (
            "unauthorized.html",
            include_str!("templates/bulma/unauthorized.html"),
        ),
        (
            "create_or_edit/checkbox.html",
            include_str!("templates/bulma/create_or_edit/checkbox.html"),
        ),
        (
            "create_or_edit/input.html",
            include_str!("templates/bulma/create_or_edit/input.html"),
        ),
        (
            "create_or_edit/selectlist.html",
            include_str!("templates/bulma/create_or_edit/selectlist.html"),
        ),
        (
            "create_or_edit/inline.html",
            include_str!("templates/bulma/create_or_edit/inline.html"),
        ),
        (
            "list/header.html",
            include_str!("templates/bulma/list/header.html"),
        ),
        (
            "list/row.html",
            include_str!("templates/bulma/list/row.html"),
        ),
        (
            "list/filter.html",
            include_str!("templates/bulma/list/filter.html"),
        ),
        (
            "card_grid.html",
            include_str!("templates/bulma/card_grid.html"),
        ),
    ];
    add_templates_to_tera(tera, TEMPLATES);
}

#[cfg(feature = "bootstrapv5_css")]
fn load_templates_into(tera: &mut Tera) {
    const TEMPLATES: &[(&str, &str)] = &[
        ("base.html", include_str!("templates/bootstrapv5/base.html")),
        ("list.html", include_str!("templates/bootstrapv5/list.html")),
        (
            "create_or_edit.html",
            include_str!("templates/bootstrapv5/create_or_edit.html"),
        ),
        ("head.html", include_str!("templates/bootstrapv5/head.html")),
        (
            "index.html",
            include_str!("templates/bootstrapv5/index.html"),
        ),
        (
            "loader.html",
            include_str!("templates/bootstrapv5/loader.html"),
        ),
        (
            "navbar.html",
            include_str!("templates/bootstrapv5/navbar.html"),
        ),
        (
            "not_found.html",
            include_str!("templates/bootstrapv5/not_found.html"),
        ),
        ("show.html", include_str!("templates/bootstrapv5/show.html")),
        (
            "unauthorized.html",
            include_str!("templates/bootstrapv5/unauthorized.html"),
        ),
        (
            "create_or_edit/checkbox.html",
            include_str!("templates/bootstrapv5/create_or_edit/checkbox.html"),
        ),
        (
            "create_or_edit/input.html",
            include_str!("templates/bootstrapv5/create_or_edit/input.html"),
        ),
        (
            "create_or_edit/selectlist.html",
            include_str!("templates/bootstrapv5/create_or_edit/selectlist.html"),
        ),
        (
            "create_or_edit/inline.html",
            include_str!("templates/bootstrapv5/create_or_edit/inline.html"),
        ),
        (
            "list/header.html",
            include_str!("templates/bootstrapv5/list/header.html"),
        ),
        (
            "list/row.html",
            include_str!("templates/bootstrapv5/list/row.html"),
        ),
        (
            "list/filter.html",
            include_str!("templates/bootstrapv5/list/filter.html"),
        ),
        (
            "card_grid.html",
            include_str!("templates/bootstrapv5/card_grid.html"),
        ),
    ];
    add_templates_to_tera(tera, TEMPLATES);
}
