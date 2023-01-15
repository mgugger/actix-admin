//! # Actix Admin Macros
//!
//! Macros used by the actix-admin crate

use proc_macro;
use quote::quote;

mod struct_fields;
use struct_fields::*;

mod selectlist_fields;
use selectlist_fields::{get_select_list_from_enum, get_select_list_from_model, get_select_lists};

mod attributes;
mod model_fields;

#[proc_macro_derive(DeriveActixAdminEnumSelectList, attributes(actix_admin))]
pub fn derive_actix_admin_enum_select_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    get_select_list_from_enum(input)
}

#[proc_macro_derive(DeriveActixAdminModelSelectList, attributes(actix_admin))]
pub fn derive_actix_admin_model_select_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    get_select_list_from_model(input)
}

#[proc_macro_derive(DeriveActixAdmin, attributes(actix_admin))]
pub fn derive_actix_admin(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expanded = quote! {
        use std::convert::From;
        use actix_admin::prelude::*;
        use sea_orm::{
            ActiveValue::Set, 
            ConnectOptions, 
            DatabaseConnection, 
            entity::*, 
            query::*,
            EntityTrait
        };
        use std::collections::HashMap;
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

        #[actix_admin::prelude::async_trait(?Send)]
        impl ActixAdminViewModelTrait for Entity {
            async fn list(db: &DatabaseConnection, page: u64, entities_per_page: u64, search: &str, sort_by: &str, sort_order: &SortOrder) -> Result<(u64, Vec<ActixAdminModel>), ActixAdminError> {
                let entities = Entity::list_model(db, page, entities_per_page, search, sort_by, sort_order).await;
                entities
            }

            fn validate_entity(model: &mut ActixAdminModel) {
                Entity::validate_model(model);

                if !model.has_errors() {
                    let active_model = ActiveModel::from(model.clone());
                    let custom_errors = Entity::validate(&active_model);
                    model.custom_errors = custom_errors;
                }
            } 

            async fn create_entity(db: &DatabaseConnection, mut model: ActixAdminModel) -> Result<ActixAdminModel, ActixAdminError> {
                let new_model = ActiveModel::from(model.clone());
                let insert_operation = Entity::insert(new_model).exec(db).await?;
                
                model.primary_key = Some(insert_operation.last_insert_id.to_string());

                Ok(model)
            }

            async fn get_entity(db: &DatabaseConnection, id: i32) -> Result<ActixAdminModel, ActixAdminError> {
                // TODO: separate primary key from other keys
                let entity = Entity::find_by_id(id).one(db).await?;
                match entity {
                    Some(e) => Ok(ActixAdminModel::from(e)),
                    _ => Err(ActixAdminError::EntityDoesNotExistError)
                }
            }

            async fn edit_entity(db: &DatabaseConnection, id: i32, mut model: ActixAdminModel) -> Result<ActixAdminModel, ActixAdminError> {
                let entity: Option<Model> = Entity::find_by_id(id).one(db).await?;

                match entity {
                    Some(e) => {
                        let mut entity: ActiveModel = e.into();
                        #(#fields_for_edit_model);*;
                        let entity: Model = entity.update(db).await?;    
                        Ok(model)        
                    },
                    _ => Err(ActixAdminError::EntityDoesNotExistError)
                }
            }

            async fn delete_entity(db: &DatabaseConnection, id: i32) -> Result<bool, ActixAdminError> {
                let result = Entity::delete_by_id(id).exec(db).await;

                match result {
                    Ok(_) => Ok(true),
                    Err(_) => Err(ActixAdminError::DeleteError)
                }
            }

            async fn get_select_lists(db: &DatabaseConnection) -> Result<HashMap<String, Vec<(String, String)>>, ActixAdminError> {
                Ok(hashmap![
                    #(#select_lists),*
                ])
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
    let fields_textarea = get_actix_admin_fields_textarea(&fields);
    let fields_file_upload = get_actix_admin_fields_file_upload(&fields);
    let fields_match_name_to_columns = get_match_name_to_column(&fields);
    let fields_list_sort_positions = get_fields_list_sort_positions(&fields);

    let expanded = quote! {
        actix_admin::prelude::lazy_static! {
            pub static ref ACTIX_ADMIN_VIEWMODEL_FIELDS: Vec<ActixAdminViewModelField> = {
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

                let fields_textareas = [
                    #(#fields_textarea),*
                ];

                let fields_fileupload = [
                    #(#fields_file_upload),*
                ];

                let list_sort_positions = [
                    #(#fields_list_sort_positions),*
                ];
                
                for (field_name, html_input_type, select_list, is_option_list, fields_type_path, is_textarea, is_file_upload, list_sort_position) in actix_admin::prelude::izip!(&field_names, &html_input_types, &field_select_lists, is_option_lists, fields_type_paths, fields_textareas, fields_fileupload, list_sort_positions) {
                
                    let select_list = select_list.replace('"', "").replace(' ', "").to_string();
                    let field_name = field_name.replace('"', "").replace(' ', "").to_string();
                    let html_input_type = html_input_type.replace('"', "").replace(' ', "").to_string();

                    vec.push(ActixAdminViewModelField {
                        field_name: field_name,
                        html_input_type: html_input_type,
                        select_list: select_list.clone(),
                        is_option: is_option_list,
                        list_sort_position: list_sort_position,
                        field_type: ActixAdminViewModelFieldType::get_field_type(fields_type_path, select_list, is_textarea, is_file_upload)
                    });
                }
                vec
            };
        }

        impl From<Model> for ActixAdminModel {
            fn from(model: Model) -> Self {
                ActixAdminModel {
                    #field_for_primary_key,
                    values: hashmap![
                        #(#fields_for_from_model),*
                    ],
                    errors: HashMap::new(),
                    custom_errors: HashMap::new(),
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

        #[actix_admin::prelude::async_trait]
        impl ActixAdminModelTrait for Entity {
            async fn list_model(db: &DatabaseConnection, page: u64, posts_per_page: u64, search: &str, sort_by: &str, sort_order: &SortOrder) -> Result<(u64, Vec<ActixAdminModel>), ActixAdminError> {
                use sea_orm::{ query::* };

                let sort_column = match sort_by {
                    #(#fields_match_name_to_columns)*
                    _ => panic!("Unknown column")
                };
                
                let query = if sort_order.eq(&SortOrder::Asc) {
                    Entity::find().order_by_asc(sort_column)
                } else {
                    Entity::find().order_by_desc(sort_column)
                };

                let paginator = query
                    .filter(
                        Condition::any()
                        #(#fields_searchable)*
                    )
                    .paginate(db, posts_per_page);
                let num_pages = paginator.num_pages().await?;
                let mut model_entities = Vec::new();
                if (num_pages == 0) { return Ok((num_pages, model_entities)) };
                let entities = paginator
                    .fetch_page(std::cmp::min(num_pages - 1, page - 1))
                    .await?;
                for entity in entities {
                    model_entities.push(
                        ActixAdminModel::from(entity)
                    );
                }
 
                Ok((num_pages, model_entities))
            }

            fn validate_model(model: &mut ActixAdminModel) {
                let mut errors = HashMap::<String, String>::new();
                #(#fields_for_validate_model);*;

                model.errors = errors;
            }

            fn get_fields() -> &'static[ActixAdminViewModelField] {
                ACTIX_ADMIN_VIEWMODEL_FIELDS.as_slice()
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
