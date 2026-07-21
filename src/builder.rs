use crate::routes::{
    create_get, create_post, delete, delete_many, download, edit_get, edit_post, index, list,
    not_found, show,
};
use crate::{
    prelude::*,
    routes::{
        bulk_action, delete_file, display_card_grid, export_csv, search,
        ActixAdminBulkActionDispatch,
    },
    ActixAdminMenuElement,
};
use actix_web::{web, Route};
use std::collections::{BTreeMap, HashMap};
use std::fs;

/// Represents a builder entity which helps generating the ActixAdmin configuration.
///
/// Prefer calling the inherent methods on `ActixAdminBuilder` directly; the
/// old `ActixAdminBuilderTrait` still exists as a compatibility shim but is
/// now just a re-export of the inherent methods and does not need to be
/// brought into scope.
pub struct ActixAdminBuilder {
    pub scopes: HashMap<String, actix_web::Scope>,
    pub custom_routes: Vec<(String, Route)>,
    pub actix_admin: ActixAdmin,
    pub custom_index: Option<Route>,
}

/// Compatibility trait for pre-0.8 code that did
/// `use actix_admin::builder::ActixAdminBuilderTrait;` before calling any
/// builder method. All the real behavior now lives as inherent methods on
/// [`ActixAdminBuilder`]; this trait simply re-exports them so existing
/// user code keeps compiling. New code should not implement it.
pub trait ActixAdminBuilderTrait {
    fn new(configuration: ActixAdminConfiguration) -> Self;
    fn get_scope(self) -> actix_web::Scope;
    fn get_actix_admin(&self) -> ActixAdmin;
}

impl ActixAdminBuilderTrait for ActixAdminBuilder {
    fn new(configuration: ActixAdminConfiguration) -> Self {
        Self::new(configuration)
    }
    fn get_scope(self) -> actix_web::Scope {
        Self::get_scope(self)
    }
    fn get_actix_admin(&self) -> ActixAdmin {
        Self::get_actix_admin(self)
    }
}

impl ActixAdminBuilder {
    pub fn new(configuration: ActixAdminConfiguration) -> Self {
        ActixAdminBuilder {
            actix_admin: ActixAdmin {
                entity_names: BTreeMap::new(),
                view_models: HashMap::new(),
                card_grids: HashMap::new(),
                configuration,
                tera: crate::tera_templates::get_tera(),
                support_path: None,
            },
            custom_routes: Vec::new(),
            scopes: HashMap::new(),
            custom_index: None,
        }
    }

    pub fn add_entity<E: ActixAdminViewModelTrait + 'static>(
        &mut self,
        view_model: &ActixAdminViewModel,
    ) {
        self.add_entity_to_category::<E>(view_model, "");
    }

    pub fn add_entity_to_category<E: ActixAdminViewModelTrait + 'static>(
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
                .route("/search", web::get().to(search::<E>))
                .route("/create", web::post().to(create_post::<E>))
                .route("/edit/{id}", web::get().to(edit_get::<E>))
                .route("/edit/{id}", web::post().to(edit_post::<E>))
                .route("/delete", web::delete().to(delete_many::<E>))
                .route("/delete/{id}", web::delete().to(delete::<E>))
                .route("/show/{id}", web::get().to(show::<E>))
                .route("/file/{id}/{column_name}", web::get().to(download::<E>))
                .route(
                    "/file/{id}/{column_name}",
                    web::delete().to(delete_file::<E>),
                )
                .default_service(web::to(not_found)),
        );

        if let Err(e) = fs::create_dir_all(format!(
            "{}/{}",
            self.actix_admin.configuration.file_upload_directory,
            E::get_entity_name()
        )) {
            // Don't panic at startup if the process lacks write permission or
            // the upload directory isn't reachable yet. Entities without file
            // fields never touch this path; entities with file fields will
            // surface the error to the user at upload time via a 500.
            log::warn!(
                "actix_admin: could not create upload directory for entity `{}`: {e}",
                E::get_entity_name()
            );
        }

        let menu_element = ActixAdminMenuElement {
            name: E::get_entity_name(),
            link: E::get_entity_name(),
            is_custom_handler: false,
        };
        self.push_menu_element(category_name, menu_element, false);

        self.actix_admin
            .view_models
            .insert(E::get_entity_name(), view_model.clone());
    }

    pub fn add_custom_handler_for_index(&mut self, route: Route) {
        self.custom_index = Some(route);
    }

    /// Register a custom bulk action on an entity. `action` is the metadata
    /// rendered in the list-page actions dropdown; the entity type `E` must
    /// provide a `run_bulk_action` implementation (via
    /// `impl ActixAdminBulkActionDispatch for Entity`) that matches on
    /// `action.name` and executes the requested work.
    pub fn add_bulk_action_for_entity<
        E: ActixAdminViewModelTrait + ActixAdminBulkActionDispatch + 'static,
    >(
        &mut self,
        action: ActixAdminBulkAction,
    ) {
        let entity_name = E::get_entity_name();
        let vm = self
            .actix_admin
            .view_models
            .get_mut(&entity_name)
            .unwrap_or_else(|| panic!("add_bulk_action_for_entity: entity `{entity_name}` must be registered via add_entity first"));
        let is_first_action = vm.bulk_actions.is_empty();
        vm.bulk_actions.push(action);

        // Register the `/action/{name}` route the first time we get an
        // action for this entity, so entities that never opt in don't have
        // to satisfy the ActixAdminBulkActionDispatch bound.
        if is_first_action {
            let scope = self
                .scopes
                .remove(&entity_name)
                .unwrap_or_else(|| web::scope(&format!("/{}", entity_name)));
            self.scopes.insert(
                entity_name,
                scope.route("/action/{name}", web::post().to(bulk_action::<E>)),
            );
        }
    }

    pub fn add_custom_handler_to_category(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        add_to_menu: bool,
        category_name: &str,
    ) {
        self.custom_routes.push((path.to_string(), route));

        if add_to_menu {
            let menu_element = ActixAdminMenuElement {
                name: menu_element_name.to_string(),
                link: path.replacen("/", "", 1),
                is_custom_handler: true,
            };
            self.push_menu_element(category_name, menu_element, true);
        }
    }

    pub fn add_card_grid(
        &mut self,
        menu_element_name: &str,
        path: &str,
        elements: Vec<Vec<String>>,
        add_to_menu: bool,
    ) {
        self.add_card_grid_to_category(menu_element_name, path, elements, add_to_menu, "");
    }

    pub fn add_card_grid_to_category(
        &mut self,
        menu_element_name: &str,
        path: &str,
        elements: Vec<Vec<String>>,
        add_to_menu: bool,
        category_name: &str,
    ) {
        self.custom_routes
            .push((path.to_string(), web::get().to(display_card_grid)));
        self.actix_admin
            .card_grids
            .insert(path.replace("/", ""), elements);

        if add_to_menu {
            let menu_element = ActixAdminMenuElement {
                name: menu_element_name.to_string(),
                link: path.replacen("/", "", 1),
                is_custom_handler: true,
            };
            self.push_menu_element(category_name, menu_element, true);
        }
    }

    pub fn add_custom_handler(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        add_to_menu: bool,
    ) {
        self.add_custom_handler_to_category(menu_element_name, path, route, add_to_menu, "");
    }

    pub fn add_support_handler(&mut self, arg: &str, support: Route) {
        self.custom_routes.push((arg.to_string(), support));
        self.actix_admin.support_path = Some(arg.replace("/", ""));
    }

    pub fn add_custom_handler_for_entity<E: ActixAdminViewModelTrait + 'static>(
        &mut self,
        menu_element_name: &str,
        path: &str,
        route: Route,
        add_to_menu: bool,
    ) {
        self.add_custom_handler_for_entity_in_category::<E>(
            menu_element_name,
            path,
            route,
            "",
            add_to_menu,
        );
    }

    pub fn add_custom_handler_for_entity_in_category<E: ActixAdminViewModelTrait + 'static>(
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

        let entity_name = E::get_entity_name();
        let scope = self
            .scopes
            .remove(&entity_name)
            .unwrap_or_else(|| web::scope(&format!("/{}", entity_name)));
        self.scopes.insert(entity_name, scope.route(path, route));

        if add_to_menu {
            if let Some(entity_list) = self.actix_admin.entity_names.get_mut(category_name) {
                if !entity_list.contains(&menu_element) {
                    entity_list.push(menu_element);
                }
            }
        }
    }

    pub fn get_scope(self) -> actix_web::Scope {
        let index_handler = self.custom_index.unwrap_or_else(|| web::get().to(index));
        let mut admin_scope = web::scope(self.actix_admin.configuration.base_path)
            .route("/", index_handler)
            .default_service(web::to(not_found));

        for (_, scope) in self.scopes {
            admin_scope = admin_scope.service(scope);
        }
        for (path, route) in self.custom_routes {
            admin_scope = admin_scope.route(&path, route);
        }
        admin_scope
    }

    pub fn get_actix_admin(&self) -> ActixAdmin {
        self.actix_admin.clone()
    }
}

impl ActixAdminBuilder {
    /// Insert `element` under `category_name` in the menu, creating the category
    /// entry if it doesn't exist. If `dedupe` is true, skip elements already present.
    fn push_menu_element(
        &mut self,
        category_name: &str,
        element: ActixAdminMenuElement,
        dedupe: bool,
    ) {
        let list = self
            .actix_admin
            .entity_names
            .entry(category_name.to_string())
            .or_default();
        if !dedupe || !list.contains(&element) {
            list.push(element);
        }
    }
}
