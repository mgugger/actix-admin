use actix_web::{ web, Route };
use std::collections::HashMap;

use crate::prelude::*;

use crate::routes::{create_get, create_post, delete, delete_many, edit_get, edit_post, index, list, show};

/// Represents a builder entity which helps generating the ActixAdmin configuration
pub struct ActixAdminBuilder {
    pub scopes: HashMap<String, actix_web::Scope>,
    pub actix_admin: ActixAdmin,
    pub custom_index: Option<Route>
}

/// The trait to work with ActixAdminBuilder
pub trait ActixAdminBuilderTrait {
    fn new(configuration: ActixAdminConfiguration) -> Self;
    fn add_entity<T: ActixAdminAppDataTrait + 'static, E: ActixAdminViewModelTrait + 'static>(
        &mut self,
        view_model: &ActixAdminViewModel,
    );
    fn add_custom_handler_for_entity<T: ActixAdminAppDataTrait + 'static, E: ActixAdminViewModelTrait + 'static>(
        &mut self,
        path: &str,
        route: Route
    );
    fn add_custom_handler_for_index<T: ActixAdminAppDataTrait + 'static>(
        &mut self,
        route: Route
    );
    fn get_scope<T: ActixAdminAppDataTrait + 'static>(self) -> actix_web::Scope;
    fn get_actix_admin(&self) -> ActixAdmin;
}

impl ActixAdminBuilderTrait for ActixAdminBuilder {
    fn new(configuration: ActixAdminConfiguration) -> Self {
        ActixAdminBuilder {
            actix_admin: ActixAdmin {
                entity_names: Vec::new(),
                view_models: HashMap::new(),
                configuration: configuration
            },
            scopes: HashMap::new(),
            custom_index: None
        }
    }

    fn add_entity<T: ActixAdminAppDataTrait + 'static, E: ActixAdminViewModelTrait + 'static>(
        &mut self,
        view_model: &ActixAdminViewModel,
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
        );

        self.actix_admin.entity_names.push(E::get_entity_name());
        let key = E::get_entity_name();
        self.actix_admin.view_models.insert(key, view_model.clone());
    }

    fn add_custom_handler_for_index<T: ActixAdminAppDataTrait + 'static>(
        &mut self,
        route: Route
    ) {
        self.custom_index = Some(route);
    }

    fn add_custom_handler_for_entity<T: ActixAdminAppDataTrait + 'static, E: ActixAdminViewModelTrait + 'static>(
        &mut self,
        path: &str,
        route: Route
    ) {
        let existing_scope = self.scopes.remove(&E::get_entity_name());
        match existing_scope {
            Some(scope) => {
                let existing_scope = scope.route(path, route);
                self.scopes.insert(E::get_entity_name(), existing_scope);
            },
            _ => {
                let new_scope = 
                    web::scope(&format!("/{}", E::get_entity_name()))
                    .route(path, route); 
                self.scopes.insert(E::get_entity_name(), new_scope);
            }
        }        

        if !self.actix_admin.entity_names.contains(&E::get_entity_name()) {
            self.actix_admin.entity_names.push(E::get_entity_name());
        }
    }

    fn get_scope<T: ActixAdminAppDataTrait + 'static>(mut self) -> actix_web::Scope {
        let index_handler = match self.custom_index {
            Some(handler) => handler,
            _ => web::get().to(index::<T>)
        };
        let mut admin_scope = web::scope("/admin").route("/", index_handler);
        for entity_name in self.actix_admin.entity_names {
            let scope = self.scopes.remove(&entity_name).unwrap();
            admin_scope = admin_scope.service(scope);
        }

        admin_scope
    }

    fn get_actix_admin(&self) -> ActixAdmin {
        self.actix_admin.clone()
    }
}
