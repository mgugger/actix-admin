use crate::{prelude::*, ActixAdminMenuElement, routes::{delete_file, export_csv}};
use actix_web::{web, Route };
use std::collections::HashMap;
use std::fs;
use crate::routes::{
    create_get, create_post, delete, delete_many, edit_get, edit_post, index, list, not_found, show, download
};

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
    fn add_entity<E: ActixAdminViewModelTrait + 'static>(
        &mut self,
        view_model: &ActixAdminViewModel,
    );
    fn add_entity_to_category<
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
    fn add_custom_handler_to_category(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        add_to_menu: bool,
        category: &str
    );
    fn add_custom_handler_for_entity<
        E: ActixAdminViewModelTrait + 'static,
    >(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        add_to_menu: bool
    );
    fn add_custom_handler_for_entity_in_category<
        E: ActixAdminViewModelTrait + 'static,
    >(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        category_name: &str,
        add_to_menu: bool,
    );
    fn add_custom_handler_for_index(&mut self, route: Route);
    fn get_scope(self) -> actix_web::Scope;
    fn get_actix_admin(&self) -> ActixAdmin;
    fn add_support_handler(&mut self, arg: &str, support: Route);
}

impl ActixAdminBuilderTrait for ActixAdminBuilder {
    fn new(configuration: ActixAdminConfiguration) -> Self {
        ActixAdminBuilder {
            actix_admin: ActixAdmin {
                entity_names: HashMap::new(),
                view_models: HashMap::new(),
                configuration: configuration,
                tera: crate::tera_templates::get_tera(),
                support_path: None
            },
            custom_routes: Vec::new(),
            scopes: HashMap::new(),
            custom_index: None,
        }
    }

    fn add_entity<E: ActixAdminViewModelTrait + 'static>(
        &mut self,
        view_model: &ActixAdminViewModel,
    ) {
        let _ = &self.add_entity_to_category::<E>(view_model, "");
    }

    fn add_entity_to_category<
        E: ActixAdminViewModelTrait + 'static,
    >(
        &mut self,
        view_model: &ActixAdminViewModel,
        category_name: &str,
    ) {
        self.scopes.insert(
            E::get_entity_name(),
            web::scope(&format!("/{}", E::get_entity_name()))
                .route("/list", web::get().to(list::<E>))
                .route("/export_csv", web::get().to(export_csv::<E>))
                .route("/create", web::get().to(create_get::<E>))
                .route("/create", web::post().to(create_post::<E>))
                .route("/edit/{id}", web::get().to(edit_get::<E>))
                .route("/edit/{id}", web::post().to(edit_post::<E>))
                .route("/delete", web::delete().to(delete_many::<E>))
                .route("/delete/{id}", web::delete().to(delete::<E>))
                .route("/show/{id}", web::get().to(show::<E>))
                .route("/file/{id}/{column_name}", web::get().to(download::<E>))
                .route("/file/{id}/{column_name}", web::delete().to(delete_file::<E>))
                .default_service(web::to(not_found))
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

    fn add_custom_handler_for_index(&mut self, route: Route) {
        self.custom_index = Some(route);
    }

    fn add_custom_handler_to_category(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        add_to_menu: bool,
        category_name: &str
    ) {
        self.custom_routes.push((path.to_string(), route));

        if add_to_menu {
            let menu_element = ActixAdminMenuElement {
                name: menu_element_name.to_string(),
                link: path.replacen("/", "", 1),
                is_custom_handler: true,
            };
            let category = self.actix_admin.entity_names.get_mut(category_name);
            match category {
                Some(entity_list) => {
                    if !entity_list.contains(&menu_element) {
                        entity_list.push(menu_element);
                    }
                }
                None => {
                    let mut entity_list = Vec::new();
                    entity_list.push(menu_element);
                    self.actix_admin
                        .entity_names
                        .insert(category_name.to_string(), entity_list);
                },
            }
        }
    }

    fn add_custom_handler(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        add_to_menu: bool
    ) {
        self.add_custom_handler_to_category(menu_element_name, path, route, add_to_menu, "");
    }

    fn add_support_handler(&mut self, arg: &str, support: Route) {
        self.custom_routes.push((arg.to_string(), support));
        self.actix_admin.support_path = Some(arg.to_string().replace("/", ""));
    }

    fn add_custom_handler_for_entity<
        E: ActixAdminViewModelTrait + 'static,
    >(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        add_to_menu: bool,
    ) {
        let _ = &self.add_custom_handler_for_entity_in_category::<E>(
            menu_element_name,
            path,
            route,
            "",
            add_to_menu,
        );
    }

    fn add_custom_handler_for_entity_in_category<
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

    fn get_scope(self) -> actix_web::Scope {
        let index_handler = match self.custom_index {
            Some(handler) => handler,
            _ => web::get().to(index),
        };
        let mut admin_scope = web::scope(&self.actix_admin.configuration.base_path)
            .route("/", index_handler)
            .default_service(web::to(not_found));

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
