use actix_web::web;
use std::collections::HashMap;

use crate::prelude::*;

use crate::routes::{create_get, create_post, delete_post, edit_get, edit_post, index, list};

pub struct ActixAdminBuilder {
    pub scopes: Vec<actix_web::Scope>,
    pub actix_admin: ActixAdmin,
}

pub trait ActixAdminBuilderTrait {
    fn new() -> Self;
    fn add_entity<T: ActixAdminAppDataTrait + 'static, E: ActixAdminViewModelTrait + 'static>(
        &mut self,
        view_model: &ActixAdminViewModel,
    );
    fn get_scope<T: ActixAdminAppDataTrait + 'static>(self) -> actix_web::Scope;
    fn get_actix_admin(&self) -> ActixAdmin;
}

impl ActixAdminBuilderTrait for ActixAdminBuilder {
    fn new() -> Self {
        ActixAdminBuilder {
            actix_admin: ActixAdmin {
                entity_names: Vec::new(),
                view_models: HashMap::new(),
            },
            scopes: Vec::new(),
        }
    }

    fn add_entity<T: ActixAdminAppDataTrait + 'static, E: ActixAdminViewModelTrait + 'static>(
        &mut self,
        view_model: &ActixAdminViewModel,
    ) {
        self.scopes.push(
            web::scope(&format!("/{}", E::get_entity_name()))
                .route("/list", web::get().to(list::<T, E>))
                .route("/create", web::get().to(create_get::<T, E>))
                .route("/create", web::post().to(create_post::<T, E>))
                .route("/edit/{id}", web::get().to(edit_get::<T, E>))
                .route("/edit/{id}", web::post().to(edit_post::<T, E>))
                .route("/delete/{id}", web::post().to(delete_post::<T, E>))
        );

        self.actix_admin.entity_names.push(E::get_entity_name());
        let key = E::get_entity_name();
        self.actix_admin.view_models.insert(key, view_model.clone());
    }

    fn get_scope<T: ActixAdminAppDataTrait + 'static>(self) -> actix_web::Scope {
        let mut scope = web::scope("/admin").route("/", web::get().to(index::<T>));
        for entity_scope in self.scopes {
            scope = scope.service(entity_scope);
        }

        scope
    }

    fn get_actix_admin(&self) -> ActixAdmin {
        self.actix_admin.clone()
    }
}
