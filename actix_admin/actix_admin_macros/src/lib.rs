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
            async fn list<T: AppDataTrait>(req: HttpRequest, data: web::Data<T>) -> Result<HttpResponse, Error> {
                let model = ActixAdminViewModel::from(Entity);
                let entity_names = &data.get_actix_admin().entity_names;

                let db = data.get_db();
                let entities = Entity::list_db(db, 1, 5);
                // TODO: Get ViewModel from ActixAdmin to honor individual settings
                actix_admin::list_model(req, &data, model, entity_names)
            }

            async fn create_get<T: AppDataTrait>(req: HttpRequest, data: web::Data<T>) -> Result<HttpResponse, Error> {
                let db = &data.get_db();
                let entity_names = &data.get_actix_admin().entity_names;
                // TODO: Get ViewModel from ActixAdmin to honor individual settings
                let model = ActixAdminViewModel::from(Entity);
                actix_admin::create_get_model(req, &data, model, entity_names)
            }

            async fn create_post<T: AppDataTrait, M>(req: HttpRequest, data: web::Data<T>, post_form: web::Form<M>) -> Result<HttpResponse, Error> {
                let view_model = ActixAdminViewModel::from(Entity);

                let form = post_form.into_inner();

                let new_model = ActiveModel {
                    title: Set("test".to_string()),
                    text: Set("test".to_string()),
                    ..Default::default()
                };
                let insert_operation = Entity::insert(new_model).exec(data.get_db()).await;            

                actix_admin::create_post_model(req, &data, view_model)
            }
        }
        #[async_trait]
        impl ActixAdminModelTrait for Entity {
            async fn list_db(db: &DatabaseConnection, page: usize, posts_per_page: usize) -> Vec<&str> {
                use sea_orm::{ query::* };
                let paginator = Entity::find()
                    .order_by_asc(Column::Id)
                    .paginate(db, posts_per_page);
                let entities = paginator
                    .fetch_page(page - 1)
                    .await
                    .expect("could not retrieve entities");
                //entities to ActixAdminModel
                vec![
                ]
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
