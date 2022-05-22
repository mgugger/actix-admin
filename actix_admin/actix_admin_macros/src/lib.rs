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
        use actix_admin::{ ActixAdminField, ActixAdminModelTrait, ActixAdminViewModelTrait, ActixAdminViewModel, ActixAdminModel, AppDataTrait };
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

        #[async_trait(?Send)]
        impl ActixAdminViewModelTrait for Entity {
            async fn list(self, db: &DatabaseConnection, page: usize, entities_per_page: usize) -> Vec<ActixAdminModel> {
                let model = ActixAdminViewModel::from(Entity);
                let entities = Entity::list_model(db, 1, 5).await;
                entities
            }

            async fn create_entity(self, db: &DatabaseConnection, model: ActixAdminModel) -> ActixAdminModel {
                let new_model = ActiveModel {
                    title: Set("test".to_string()),
                    text: Set("test".to_string()),
                    ..Default::default()
                };
                let insert_operation = Entity::insert(new_model).exec(db).await;

                ActixAdminModel{ values: HashMap::new() }
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
                    let mut model_values = HashMap::new();
                    model_values.insert("title", entity.title);
                    model_values.insert("text", entity.text);
                    model_values.insert("id", entity.id.to_string());
                    model_entities.push(
                        ActixAdminModel {
                            values: model_values,
                        });
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
