use proc_macro;
use quote::quote;

mod struct_fields;
use struct_fields::{
    get_actix_admin_fields, get_actix_admin_fields_html_input,
    get_actix_admin_fields_is_option_list, get_actix_admin_fields_searchable,
    get_actix_admin_fields_select_list, get_actix_admin_fields_type_path_string,
    get_field_for_primary_key, get_fields_for_create_model, get_fields_for_edit_model,
    get_fields_for_from_model, get_fields_for_tokenstream, get_fields_for_validate_model,
    get_primary_key_field_name,
};

mod selectlist_fields;
use selectlist_fields::{get_select_list, get_select_lists};

mod attributes;
mod model_fields;

#[proc_macro_derive(DeriveActixAdminSelectList, attributes(actix_admin))]
pub fn derive_actix_admin_select_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    get_select_list(input)
}

#[proc_macro_derive(DeriveActixAdmin, attributes(actix_admin))]
pub fn derive_actix_admin(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expanded = quote! {
        use std::convert::From;
        use async_trait::async_trait;
        use actix_admin::prelude::*;
        use sea_orm::ActiveValue::Set;
        use sea_orm::{ConnectOptions, DatabaseConnection};
        use sea_orm::{entity::*, query::*};
        use std::collections::HashMap;
        use sea_orm::EntityTrait;
        use itertools::izip;
        use actix_session::{Session};
    };
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(DeriveActixAdminViewModel, attributes(actix_admin))]
pub fn derive_actix_admin_view_model(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let fields = get_fields_for_tokenstream(input);

    let name_primary_field_str = get_primary_key_field_name(&fields);
    let fields_for_edit_model = get_fields_for_edit_model(&fields);
    let fields_searchable = get_actix_admin_fields_searchable(&fields);
    let has_searchable_fields = fields_searchable.len() > 0;

    let select_lists = get_select_lists(&fields);

    let expanded = quote! {
        impl From<Entity> for ActixAdminViewModel {
            fn from(entity: Entity) -> Self {
                ActixAdminViewModel {
                    primary_key: #name_primary_field_str.to_string(),
                    entity_name: entity.table_name().to_string(),
                    fields: Entity::get_fields(),
                    show_search: #has_searchable_fields,
                    user_can_access: None
                }
            }
        }

        #[async_trait(?Send)]
        impl ActixAdminViewModelTrait for Entity {
            async fn list(db: &DatabaseConnection, page: usize, entities_per_page: usize, search: &String) -> (usize, Vec<ActixAdminModel>) {
                let entities = Entity::list_model(db, page, entities_per_page, search).await;
                entities
            }

            async fn create_entity(db: &DatabaseConnection, mut model: ActixAdminModel) -> ActixAdminModel {
                let mut validation_errs = Entity::validate_model(&model);
                //model.errors.append(&mut validation_errs);
                model.errors = validation_errs;

                if !model.has_errors() {
                    let new_model = ActiveModel::from(model.clone());
                    let insert_operation = Entity::insert(new_model).exec(db).await;
                }

                model
            }

            async fn get_entity(db: &DatabaseConnection, id: i32) -> ActixAdminModel {
                // TODO: separate primary key from other keys
                let entity = Entity::find_by_id(id).one(db).await.unwrap().unwrap();
                let model = ActixAdminModel::from(entity);
                model
            }

            async fn edit_entity(db: &DatabaseConnection, id: i32, mut model: ActixAdminModel) -> ActixAdminModel {
                let mut validation_errs = Entity::validate_model(&model);
                //model.errors.append(&mut validation_errs);
                model.errors=validation_errs;

                if !model.has_errors() {
                    let entity: Option<Model> = Entity::find_by_id(id).one(db).await.unwrap();
                    let mut entity: ActiveModel = entity.unwrap().into();

                    #(#fields_for_edit_model);*;
                    let entity: Model = entity.update(db).await.unwrap();
                }
                model
            }

            async fn delete_entity(db: &DatabaseConnection, id: i32) -> bool {
                let result = Entity::delete_by_id(id).exec(db).await;

                match result {
                    Ok(_) => true,
                    Err(_) => false
                }
            }

            async fn get_select_lists(db: &DatabaseConnection) -> HashMap<String, Vec<(String, String)>> {
                hashmap![
                    #(#select_lists),*
                ]
            }

            fn get_entity_name() -> String {
                Entity.table_name().to_string()
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(DeriveActixAdminModel, attributes(actix_admin))]
pub fn derive_actix_admin_model(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let fields = get_fields_for_tokenstream(input);

    let field_names = get_actix_admin_fields(&fields);
    let field_html_input_type = get_actix_admin_fields_html_input(&fields);
    let field_select_list = get_actix_admin_fields_select_list(&fields);
    let is_option_list = get_actix_admin_fields_is_option_list(&fields);
    let fields_for_create_model = get_fields_for_create_model(&fields);
    let fields_for_from_model = get_fields_for_from_model(&fields);
    let field_for_primary_key = get_field_for_primary_key(&fields);
    let fields_for_validate_model = get_fields_for_validate_model(&fields);
    let fields_searchable = get_actix_admin_fields_searchable(&fields);
    let fields_type_path = get_actix_admin_fields_type_path_string(&fields);

    let expanded = quote! {
        impl From<Model> for ActixAdminModel {
            fn from(model: Model) -> Self {
                ActixAdminModel {
                    #field_for_primary_key,
                    values: hashmap![
                        #(#fields_for_from_model),*
                    ],
                    errors: HashMap::new()
                }
            }
        }

        impl From<ActixAdminModel> for ActiveModel {
            fn from(model: ActixAdminModel) -> Self {
                ActiveModel
                {
                    #(#fields_for_create_model),*
                    ,
                    ..Default::default()
                }
            }
        }

        #[async_trait]
        impl ActixAdminModelTrait for Entity {
            async fn list_model(db: &DatabaseConnection, page: usize, posts_per_page: usize, search: &String) -> (usize, Vec<ActixAdminModel>) {
                use sea_orm::{ query::* };
                let paginator = Entity::find()
                    .filter(
                        Condition::any()
                        #(#fields_searchable)*
                    )
                    .order_by_asc(Column::Id)
                    .paginate(db, posts_per_page);
                let num_pages = paginator.num_pages().await.ok().unwrap();
                let entities = paginator
                    .fetch_page(page - 1)
                    .await
                    .expect("could not retrieve entities");
                let mut model_entities = Vec::new();
                for entity in entities {
                    model_entities.push(
                        ActixAdminModel::from(entity)
                    );
                }

                (num_pages, model_entities)
            }

            fn validate_model(model: &ActixAdminModel) -> HashMap<String, String> {
                let mut errors = HashMap::<String, String>::new();
                #(#fields_for_validate_model);*;
                //let mut custom_errors = Entity.validate();
                //errors.append(&mut custom_errors);
                errors
            }

            fn get_fields() -> Vec<ActixAdminViewModelField> {
                let mut vec = Vec::new();
                
                let field_names = stringify!(
                        #(#field_names),*
                ).split(",")
                .collect::<Vec<_>>();

                let html_input_types = stringify!(
                    #(#field_html_input_type),*
                ).split(",")
                .collect::<Vec<_>>();

                let field_select_lists = stringify!(
                    #(#field_select_list),*
                ).split(",")
                .collect::<Vec<_>>();

                let is_option_lists = [
                    #(#is_option_list),*
                ];

                let fields_type_paths = [
                    #(#fields_type_path),*
                ];

                for (field_name, html_input_type, select_list, is_option_list, fields_type_path) in izip!(&field_names, &html_input_types, &field_select_lists, is_option_lists, fields_type_paths) {
                    
                    let select_list = select_list.replace('"', "").replace(' ', "").to_string();
                    let field_name = field_name.replace('"', "").replace(' ', "").to_string();
                    let html_input_type = html_input_type.replace('"', "").replace(' ', "").to_string();

                    vec.push(ActixAdminViewModelField {
                        field_name: field_name,
                        html_input_type: html_input_type,
                        select_list: select_list.clone(),
                        is_option: is_option_list,
                        field_type: ActixAdminViewModelFieldType::get_field_type(fields_type_path, select_list)
                    });
                }
                vec
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
