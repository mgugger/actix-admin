use std::{collections::HashMap, hash::BuildHasher};
use tera::Tera;
use tera::{to_value, try_get_value, Result};

use crate::view_model::{ActixAdminViewModelField, ActixAdminViewModelFieldType};

struct TeraTemplate {
    // Pages
    list_html: &'static str,
    create_or_edit_html: &'static str,
    base_html: &'static str,
    head_html: &'static str,
    index_html: &'static str,
    loader_html: &'static str,
    navbar_html: &'static str,
    not_found_html: &'static str,
    show_html: &'static str,
    unauthorized_html: &'static str,
    // create or edit
    checkbox_html: &'static str,
    input_html: &'static str,
    selectlist_html: &'static str,
    create_or_edit_inline_html: &'static str,
    // list
    list_header_html: &'static str,
    list_row_html: &'static str,
    list_filter_html: &'static str,
}

pub fn get_tera() -> Tera {
    let mut tera = load_templates();

    tera.register_filter("get_html_input_type", get_html_input_type);
    tera.register_filter("get_html_input_class", get_html_input_class);
    tera.register_filter("get_icon", get_icon);
    tera.register_filter("get_regex_val", get_regex_val);
    tera.register_filter("shorten", shorten_filter);
    tera
}

fn shorten_filter<S: BuildHasher>(
    value: &tera::Value,
    args: &HashMap<String, tera::Value, S>,
) -> Result<tera::Value> {
    let max_length = args
        .get("max_length")
        .and_then(|v| v.as_number())
        .and_then(|s| s.as_u64());
    let input = value.as_str().unwrap();

    if let Some(max) = max_length {
        if input.len() <= max as usize {
            Ok(tera::Value::String(input.to_string()))
        } else {
            Ok(tera::Value::String(
                input.chars().take(max as usize).collect(),
            ))
        }
    } else {
        Ok(tera::Value::String(input.to_string()))
    }
}

fn get_html_input_class<S: BuildHasher>(
    value: &tera::Value,
    _: &HashMap<String, tera::Value, S>,
) -> Result<tera::Value> {
    let field = try_get_value!(
        "get_html_input_class",
        "value",
        ActixAdminViewModelField,
        value
    );
    let html_input_type = match field.field_type {
        ActixAdminViewModelFieldType::TextArea => "textarea",
        ActixAdminViewModelFieldType::Checkbox => "checkbox",
        _ => "input",
    };

    Ok(to_value(html_input_type).unwrap())
}

fn get_icon<S: BuildHasher>(
    value: &tera::Value,
    _: &HashMap<String, tera::Value, S>,
) -> Result<tera::Value> {
    let field = try_get_value!("get_icon", "value", String, value);
    let font_awesome_icon = match field.as_str() {
        "true" => "<i class=\"fa-solid fa-check\"></i>",
        "false" => "<i class=\"fa-solid fa-xmark\"></i>",
        _ => panic!("not implemented icon"),
    };

    Ok(to_value(font_awesome_icon).unwrap())
}

fn get_regex_val<S: BuildHasher>(
    value: &tera::Value,
    args: &HashMap<String, tera::Value, S>,
) -> Result<tera::Value> {
    let field = try_get_value!("get_regex_val", "value", ActixAdminViewModelField, value);

    let s = args.get("values");
    let field_val = s.unwrap().get(&field.field_name);

    // println!(
    //     "field {} regex {:?}",
    //     field.field_name, field.list_regex_mask
    // );
    match (field_val, field.list_regex_mask) {
        (Some(val), Some(r)) => {
            let val_str = val.to_string();
            //let is_match = r.is_match(&val_str);
            //println!("is match: {}, regex {}", is_match, r.to_string());
            let result_str = r.replace_all(&val_str, "*");
            return Ok(to_value(result_str).unwrap());
        }
        (Some(val), None) => {
            return Ok(to_value(val).unwrap());
        }
        (_, _) => panic!("key {} not found in model values", &field.field_name),
    }
}

fn get_html_input_type<S: BuildHasher>(
    value: &tera::Value,
    _: &HashMap<String, tera::Value, S>,
) -> Result<tera::Value> {
    let field = try_get_value!(
        "get_html_input_type",
        "value",
        ActixAdminViewModelField,
        value
    );

    // TODO: convert to option
    if field.html_input_type != "" {
        return Ok(to_value(field.html_input_type).unwrap());
    }

    let html_input_type = match field.field_type {
        ActixAdminViewModelFieldType::Text => "text",
        ActixAdminViewModelFieldType::DateTime => "datetime-local",
        ActixAdminViewModelFieldType::Date => "date",
        ActixAdminViewModelFieldType::Checkbox => "checkbox",
        ActixAdminViewModelFieldType::FileUpload => "file",
        _ => "text",
    };

    Ok(to_value(html_input_type).unwrap())
}

fn add_templates_to_tera(tera: &mut Tera, tera_template: TeraTemplate) {
    let _res = tera.add_raw_templates(vec![
        ("base.html", tera_template.base_html),
        ("list.html", tera_template.list_html),
        ("create_or_edit.html", tera_template.create_or_edit_html),
        ("head.html", tera_template.head_html),
        ("index.html", tera_template.index_html),
        ("loader.html", tera_template.loader_html),
        ("navbar.html", tera_template.navbar_html),
        ("not_found.html", tera_template.not_found_html),
        ("show.html", tera_template.show_html),
        ("unauthorized.html", tera_template.unauthorized_html),
        // create or edit
        ("create_or_edit/checkbox.html", tera_template.checkbox_html),
        ("create_or_edit/input.html", tera_template.input_html),
        ("create_or_edit/selectlist.html", tera_template.selectlist_html),
        ("create_or_edit/inline.html", tera_template.create_or_edit_inline_html),
        // list
        ("list/header.html", tera_template.list_header_html),
        ("list/row.html", tera_template.list_row_html),
        ("list/filter.html", tera_template.list_filter_html),
    ]);
}

#[cfg(all(feature = "bootstrapv5_css", feature = "bulma_css"))]
compile_error!("feature \"foo\" and feature \"bar\" cannot be enabled at the same time");

// Cargo Features for CSS
#[cfg(feature = "bulma_css")]
fn load_templates() -> Tera {
    let mut tera = Tera::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/templates/bulma/*.html"
    ))
    .unwrap();
    let tera_template = TeraTemplate {
        list_html: include_str!("templates/bulma/list.html"),
        create_or_edit_html: include_str!("templates/bulma/create_or_edit.html"),
        base_html: include_str!("templates/bulma/base.html"),
        head_html: include_str!("templates/bulma/head.html"),
        index_html: include_str!("templates/bulma/index.html"),
        loader_html: include_str!("templates/bulma/loader.html"),
        navbar_html: include_str!("templates/bulma/navbar.html"),
        not_found_html: include_str!("templates/bulma/not_found.html"),
        show_html: include_str!("templates/bulma/show.html"),
        unauthorized_html: include_str!("templates/bulma/unauthorized.html"),
        // create or edit
        checkbox_html: include_str!("templates/bulma/create_or_edit/checkbox.html"),
        input_html: include_str!("templates/bulma/create_or_edit/input.html"),
        selectlist_html: include_str!("templates/bulma/create_or_edit/selectlist.html"),
        create_or_edit_inline_html: include_str!("templates/bulma/create_or_edit/inline.html"),
        // list
        list_header_html: include_str!("templates/bulma/list/header.html"),
        list_row_html: include_str!("templates/bulma/list/row.html"),
        list_filter_html: include_str!("templates/bulma/list/filter.html"),
    };

    add_templates_to_tera(&mut tera, tera_template);

    tera
}

#[cfg(feature = "bootstrapv5_css")]
fn load_templates() -> Tera {
    let mut tera = Tera::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/templates/bootstrapv5/*.html"
    ))
    .unwrap();
    let tera_template = TeraTemplate {
        list_html: include_str!("templates/bootstrapv5/list.html"),
        create_or_edit_html: include_str!("templates/bootstrapv5/create_or_edit.html"),
        base_html: include_str!("templates/bootstrapv5/base.html"),
        head_html: include_str!("templates/bootstrapv5/head.html"),
        index_html: include_str!("templates/bootstrapv5/index.html"),
        loader_html: include_str!("templates/bootstrapv5/loader.html"),
        navbar_html: include_str!("templates/bootstrapv5/navbar.html"),
        not_found_html: include_str!("templates/bootstrapv5/not_found.html"),
        show_html: include_str!("templates/bootstrapv5/show.html"),
        unauthorized_html: include_str!("templates/bootstrapv5/unauthorized.html"),
        // create or edit
        checkbox_html: include_str!("templates/bootstrapv5/create_or_edit/checkbox.html"),
        input_html: include_str!("templates/bootstrapv5/create_or_edit/input.html"),
        selectlist_html: include_str!("templates/bootstrapv5/create_or_edit/selectlist.html"),
        create_or_edit_inline_html: include_str!("templates/bootstrapv5/create_or_edit/inline.html"),
        // list
        list_header_html: include_str!("templates/bootstrapv5/list/header.html"),
        list_row_html: include_str!("templates/bootstrapv5/list/row.html"),
        list_filter_html: include_str!("templates/bootstrapv5/list/filter.html"),
    };

    add_templates_to_tera(&mut tera, tera_template);

    tera
}
