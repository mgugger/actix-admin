use proc_macro;
use quote::quote;

mod struct_fields;
use struct_fields::get_fields_for_tokenstream;

#[proc_macro_derive(DeriveActixAdminModel)]
pub fn derive_crud_fns(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let fields = get_fields_for_tokenstream(input);

    let names_const_fields_str = fields.iter().map(|(_vis, ident)| {
        let ident_name = ident.to_string();
        quote! {
            #ident_name
        }
    });

    let expanded = quote! {
        use std::convert::From;
        use async_trait::async_trait;
        use actix_web::{web, HttpResponse, HttpRequest, Error};
        use actix_admin::{ ActixAdminField, ActixAdminModelTrait, ActixAdminViewModelTrait, ActixAdminViewModel, ActixAdminModel, AppDataTrait , hashmap};
        use sea_orm::ActiveValue::Set;
        use sea_orm::{ConnectOptions, DatabaseConnection};
        use sea_orm::{entity::*, query::*};
        use std::collections::HashMap;
        use sea_orm::EntityTrait;

        impl From<Entity> for ActixAdminViewModel {
            fn from(entity: Entity) -> Self {
                ActixAdminViewModel {
                    entity_name: entity.table_name().to_string(),
                    fields: Entity::get_fields()
                }
            }
        }

        impl From<Model> for ActixAdminModel {
            fn from(model: Model) -> Self {
                ActixAdminModel {
                    // TODO: create dynamically
                    values: hashmap![
                        "title" => model.title,
                        "text" => model.text,
                        "id" => model.id.to_string()
                    ]
                }
            }
        }

        #[async_trait(?Send)]
        impl ActixAdminViewModelTrait for Entity {
            async fn list(db: &DatabaseConnection, page: usize, entities_per_page: usize) -> Vec<ActixAdminModel> {
                let entities = Entity::list_model(db, 1, 5).await;
                entities
            }

            async fn create_entity(db: &DatabaseConnection, mut model: ActixAdminModel) -> ActixAdminModel {
                let new_model = ActiveModel {
                    title: Set("test".to_string()),
                    text: Set("test".to_string()),
                    ..Default::default()
                };
                let insert_operation = Entity::insert(new_model).exec(db).await;

                model
            }
        }

        #[async_trait]
        impl ActixAdminModelTrait for Entity {
            async fn list_model(db: &DatabaseConnection, page: usize, posts_per_page: usize) -> Vec<ActixAdminModel> {
                use sea_orm::{ query::* };
                let paginator = Entity::find()
                    .order_by_asc(Column::Id)
                    .paginate(db, posts_per_page);
                let entities = paginator
                    .fetch_page(page - 1)
                    .await
                    .expect("could not retrieve entities");
                // TODO: must be dynamic
                let mut model_entities = Vec::new();
                for entity in entities {
                    model_entities.push(
                        ActixAdminModel::from(entity)
                    );
                }

                model_entities
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
