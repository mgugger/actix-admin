use crate::{prelude::*, ActixAdminMenuElement, routes::delete_file};
use actix_web::{web, Route };
use tera::Tera;
use std::collections::HashMap;
use std::fs;
use crate::routes::{
    create_get, create_post, delete, delete_many, edit_get, edit_post, index, list, not_found, show, download
};
use std::hash::BuildHasher;
use tera::{to_value, try_get_value, Result};

/// Represents a builder entity which helps generating the ActixAdmin configuration
pub struct ActixAdminBuilder {
    pub scopes: HashMap<String, actix_web::Scope>,
    pub custom_routes: Vec<(String, Route)>,
    pub actix_admin: ActixAdmin,
    pub custom_index: Option<Route>,
}

/// The trait to work with ActixAdminBuilder
pub trait ActixAdminBuilderTrait {
    fn new(configuration: ActixAdminConfiguration) -> Self;
    fn add_entity<T: ActixAdminAppDataTrait + 'static, E: ActixAdminViewModelTrait + 'static>(
        &mut self,
        view_model: &ActixAdminViewModel,
    );
    fn add_entity_to_category<
        T: ActixAdminAppDataTrait + 'static,
        E: ActixAdminViewModelTrait + 'static,
    >(
        &mut self,
        view_model: &ActixAdminViewModel,
        category_name: &str,
    );
    fn add_custom_handler(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        add_to_menu: bool,
    );
    fn add_custom_handler_for_entity<
        T: ActixAdminAppDataTrait + 'static,
        E: ActixAdminViewModelTrait + 'static,
    >(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        add_to_menu: bool
    );
    fn add_custom_handler_for_entity_in_category<
        T: ActixAdminAppDataTrait + 'static,
        E: ActixAdminViewModelTrait + 'static,
    >(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        category_name: &str,
        add_to_menu: bool,
    );
    fn add_custom_handler_for_index<T: ActixAdminAppDataTrait + 'static>(&mut self, route: Route);
    fn get_scope<T: ActixAdminAppDataTrait + 'static>(self) -> actix_web::Scope;
    fn get_actix_admin(&self) -> ActixAdmin;
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
    
    println!("field {} regex {:?}", field.field_name, field.list_regex_mask);
    match (field_val, field.list_regex_mask) {
        (Some(val), Some(r)) => {
            let val_str = val.to_string();
            let is_match = r.is_match(&val_str);
            println!("is match: {}, regex {}", is_match, r.to_string());
            let result_str = r.replace_all(&val_str, "*");
            return Ok(to_value(result_str).unwrap());
        },
        (Some(val), None) => { return Ok(to_value(val).unwrap()); },
        (_, _) => panic!("key {} not found in model values", &field.field_name)
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

fn get_tera() -> Tera {
    let mut tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "*")).unwrap();
    tera.register_filter("get_html_input_type", get_html_input_type);
    tera.register_filter("get_html_input_class", get_html_input_class);
    tera.register_filter("get_icon", get_icon);
    tera.register_filter("get_regex_val", get_regex_val);

    let list_html = include_str!("templates/list.html");
    let create_or_edit_html = include_str!("templates/create_or_edit.html");
    let base_html = include_str!("templates/base.html");
    let head_html = include_str!("templates/head.html");
    let index_html = include_str!("templates/index.html");
    let loader_html = include_str!("templates/loader.html");
    let navbar_html = include_str!("templates/navbar.html");
    let not_found_html = include_str!("templates/not_found.html");
    let show_html = include_str!("templates/show.html");
    let unauthorized_html = include_str!("templates/unauthorized.html");

    // form elements
    let checkbox_html = include_str!("templates/form_elements/checkbox.html");
    let input_html = include_str!("templates/form_elements/input.html");
    let selectlist_html = include_str!("templates/form_elements/selectlist.html");

    let _res = tera.add_raw_templates(vec![
        ("base.html", base_html),
        ("list.html", list_html),
        ("create_or_edit.html", create_or_edit_html),
        ("head.html", head_html),
        ("index.html", index_html),
        ("loader.html", loader_html),
        ("navbar.html", navbar_html),
        ("not_found.html", not_found_html),
        ("show.html",show_html),
        ("unauthorized.html", unauthorized_html),
        // form elements
        ("form_elements/checkbox.html", checkbox_html),
        ("form_elements/input.html", input_html),
        ("form_elements/selectlist.html", selectlist_html),
    ]);

    tera
}

impl ActixAdminBuilderTrait for ActixAdminBuilder {
    fn new(configuration: ActixAdminConfiguration) -> Self {
        ActixAdminBuilder {
            actix_admin: ActixAdmin {
                entity_names: HashMap::new(),
                view_models: HashMap::new(),
                configuration: configuration,
                tera: get_tera()
            },
            custom_routes: Vec::new(),
            scopes: HashMap::new(),
            custom_index: None,
        }
    }

    fn add_entity<T: ActixAdminAppDataTrait + 'static, E: ActixAdminViewModelTrait + 'static>(
        &mut self,
        view_model: &ActixAdminViewModel,
    ) {
        let _ = &self.add_entity_to_category::<T, E>(view_model, "");
    }

    fn add_entity_to_category<
        T: ActixAdminAppDataTrait + 'static,
        E: ActixAdminViewModelTrait + 'static,
    >(
        &mut self,
        view_model: &ActixAdminViewModel,
        category_name: &str,
    ) {
        self.scopes.insert(
            E::get_entity_name(),
            web::scope(&format!("/{}", E::get_entity_name()))
                .route("/list", web::get().to(list::<T, E>))
                .route("/create", web::get().to(create_get::<T, E>))
                .route("/create", web::post().to(create_post::<T, E>))
                .route("/edit/{id}", web::get().to(edit_get::<T, E>))
                .route("/edit/{id}", web::post().to(edit_post::<T, E>))
                .route("/delete", web::delete().to(delete_many::<T, E>))
                .route("/delete/{id}", web::delete().to(delete::<T, E>))
                .route("/show/{id}", web::get().to(show::<T, E>))
                .route("/file/{id}/{column_name}", web::get().to(download::<T, E>))
                .route("/file/{id}/{column_name}", web::delete().to(delete_file::<T, E>))
                .default_service(web::to(not_found::<T>))
            );

        fs::create_dir_all(format!("{}/{}", &self.actix_admin.configuration.file_upload_directory, E::get_entity_name())).unwrap();

        let category = self.actix_admin.entity_names.get_mut(category_name);
        let menu_element = ActixAdminMenuElement {
            name: E::get_entity_name(),
            link: E::get_entity_name(),
            is_custom_handler: false,
        };
        match category {
            Some(entity_list) => entity_list.push(menu_element),
            None => {
                let mut entity_list = Vec::new();
                entity_list.push(menu_element);
                self.actix_admin
                    .entity_names
                    .insert(category_name.to_string(), entity_list);
            }
        }

        let key = E::get_entity_name();
        self.actix_admin.view_models.insert(key, view_model.clone());
    }

    fn add_custom_handler_for_index<T: ActixAdminAppDataTrait + 'static>(&mut self, route: Route) {
        self.custom_index = Some(route);
    }

    fn add_custom_handler(
            &mut self,
            menu_element_name: &str,
            path: &str,
            route: Route,
            add_to_menu: bool,
        ) {
            self.custom_routes.push((path.to_string(), route));

            if add_to_menu {
                let menu_element = ActixAdminMenuElement {
                    name: menu_element_name.to_string(),
                    link: path.replacen("/", "", 1),
                    is_custom_handler: true,
                };
                let category = self.actix_admin.entity_names.get_mut("");
                match category {
                    Some(entity_list) => {
                        if !entity_list.contains(&menu_element) {
                            entity_list.push(menu_element);
                        }
                    }
                    _ => (),
                }
            }
    }

    fn add_custom_handler_for_entity<
        T: ActixAdminAppDataTrait + 'static,
        E: ActixAdminViewModelTrait + 'static,
    >(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        add_to_menu: bool,
    ) {
        let _ = &self.add_custom_handler_for_entity_in_category::<T, E>(
            menu_element_name,
            path,
            route,
            "",
            add_to_menu,
        );
    }

    fn add_custom_handler_for_entity_in_category<
        T: ActixAdminAppDataTrait + 'static,
        E: ActixAdminViewModelTrait + 'static,
    >(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        category_name: &str,
        add_to_menu: bool,
    ) {
        let menu_element = ActixAdminMenuElement {
            name: menu_element_name.to_string(),
            link: format!("{}{}", E::get_entity_name(), path),
            is_custom_handler: true,
        };

        let existing_scope = self.scopes.remove(&E::get_entity_name());

        match existing_scope {
            Some(scope) => {
                let existing_scope = scope.route(path, route);
                self.scopes
                    .insert(E::get_entity_name(), existing_scope);
            }
            _ => {
                let new_scope =
                    web::scope(&format!("/{}", E::get_entity_name())).route(path, route);
                self.scopes.insert(E::get_entity_name(), new_scope);
            }
        }

        if add_to_menu {
            let category = self.actix_admin.entity_names.get_mut(category_name);
            match category {
                Some(entity_list) => {
                    if !entity_list.contains(&menu_element) {
                        entity_list.push(menu_element);
                    }
                }
                _ => (),
            }
        }
    }

    fn get_scope<T: ActixAdminAppDataTrait + 'static>(self) -> actix_web::Scope {
        let index_handler = match self.custom_index {
            Some(handler) => handler,
            _ => web::get().to(index::<T>),
        };
        let mut admin_scope = web::scope("/admin")
            .route("/", index_handler)
            .default_service(web::to(not_found::<T>));

        for (_entity, scope) in self.scopes {
            admin_scope = admin_scope.service(scope);
        }

        for (path, route) in self.custom_routes {
            admin_scope = admin_scope.route(&path, route);
        }

        admin_scope
    }

    fn get_actix_admin(&self) -> ActixAdmin {
        self.actix_admin.clone()
    }
}