use proc_macro;
use quote::quote;

mod struct_fields;
use struct_fields::{ get_fields_for_tokenstream, get_fields_for_edit_model, get_fields_for_from_model, get_fields_for_create_model, get_field_names, get_field_for_primary_key, get_primary_key_field_name};

mod model_fields;

mod attributes;

#[proc_macro_derive(DeriveActixAdminModel, attributes(actix_admin))]
pub fn derive_crud_fns(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let fields = get_fields_for_tokenstream(input);

    let names_const_fields_str = get_field_names(&fields);
    let name_primary_field_str = get_primary_key_field_name(&fields);
    let fields_for_create_model = get_fields_for_create_model(&fields);
    let fields_for_edit_model = get_fields_for_edit_model(&fields);
    let fields_for_from_model = get_fields_for_from_model(&fields);
    let field_for_primary_key = get_field_for_primary_key(&fields);

    let expanded = quote! {
        use std::convert::From;
        use async_trait::async_trait;
        use actix_web::{web, HttpResponse, HttpRequest, Error};
        use actix_admin::prelude::*;
        use sea_orm::ActiveValue::Set;
        use sea_orm::{ConnectOptions, DatabaseConnection};
        use sea_orm::{entity::*, query::*};
        use std::collections::HashMap;
        use sea_orm::EntityTrait;
        use quote::quote;

        impl From<Entity> for ActixAdminViewModel {
            fn from(entity: Entity) -> Self {
                ActixAdminViewModel {
                    primary_key: #name_primary_field_str.to_string(),
                    entity_name: entity.table_name().to_string(),
                    fields: Entity::get_fields()
                }
            }
        }

        impl From<Model> for ActixAdminModel {
            fn from(model: Model) -> Self {
                ActixAdminModel {
                    #field_for_primary_key,
                    values: hashmap![
                        #(#fields_for_from_model),*
                    ]
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

        #[async_trait(?Send)]
        impl ActixAdminViewModelTrait for Entity {
            async fn list(db: &DatabaseConnection, page: usize, entities_per_page: usize) -> (usize, Vec<ActixAdminModel>) {
                let entities = Entity::list_model(db, page, entities_per_page).await;
                entities
            }

            async fn create_entity(db: &DatabaseConnection, mut model: ActixAdminModel) -> ActixAdminModel {
                let new_model = ActiveModel::from(model.clone());
                let insert_operation = Entity::insert(new_model).exec(db).await;

                model
            }

            async fn get_entity(db: &DatabaseConnection, id: i32) -> ActixAdminModel {
                // TODO: separate primary key from other keys
                let entity = Entity::find_by_id(id).one(db).await.unwrap().unwrap();
                let model = ActixAdminModel::from(entity);
                
                model
            }

            async fn edit_entity(db: &DatabaseConnection, id: i32, mut model: ActixAdminModel) -> ActixAdminModel {
                // TODO: separate primary key from other keys
                let entity: Option<Model> = Entity::find_by_id(id).one(db).await.unwrap();
                let mut entity: ActiveModel = entity.unwrap().into();

                #(#fields_for_edit_model);*;
                
                let entity: Model = entity.update(db).await.unwrap();
                
                model
            }

            async fn delete_entity(db: &DatabaseConnection, id: i32) -> bool {
                // TODO: separate primary key from other keys
                let entity = Entity::find_by_id(id).one(db).await.unwrap().unwrap();
                let result = entity.delete(db).await;

                match result {
                    Ok(_) => true,
                    Err(_) => false
                }
            }

            fn get_entity_name() -> String {
                Entity.table_name().to_string()
            }
        }

        #[async_trait]
        impl ActixAdminModelTrait for Entity {
            async fn list_model(db: &DatabaseConnection, page: usize, posts_per_page: usize) -> (usize, Vec<ActixAdminModel>) {
                use sea_orm::{ query::* };
                let paginator = Entity::find()
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

            fn get_fields() -> Vec<(String, ActixAdminField)> {
                let mut vec = Vec::new();
                let field_names = stringify!(
                        #(#names_const_fields_str),*
                    ).split(",")
                    .collect::<Vec<_>>()
                    .into_iter()
                    .for_each( |field_name|
                        vec.push((
                            field_name
                            .replace('"', "")
                            .replace(' ', "")
                            .to_string(),
                            // TODO: match correct ActixAdminField Value
                            ActixAdminField::Text
                            )
                        )
                );
                vec
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
