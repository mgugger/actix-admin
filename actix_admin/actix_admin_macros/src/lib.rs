use proc_macro;
use quote::quote;

#[proc_macro_derive(DeriveActixAdminModel)]
pub fn derive_crud_fns(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expanded = quote! {
        use std::convert::From;
        use async_trait::async_trait;
        use actix_web::{web, HttpResponse, HttpRequest, Error};
        use actix_admin::{ ActixAdminModelTrait, ActixAdminViewModelTrait, ActixAdminViewModel, ActixAdminModel, AppDataTrait };

        impl From<Entity> for ActixAdminViewModel {
            fn from(entity: Entity) -> Self {
                ActixAdminViewModel {
                    entity_name: entity.table_name().to_string()
                }
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
        }
        
        #[async_trait(?Send)]
        impl ActixAdminViewModelTrait for Entity {
            async fn list<T: AppDataTrait>(req: HttpRequest, data: web::Data<T>) -> Result<HttpResponse, Error> {
                let db = &data.get_db();
                let entities = Entity::list_db(db, 1, 5);
                let entity_names = &data.get_actix_admin().entity_names;
                let model = ActixAdminViewModel::from(Entity);
                actix_admin::list_model(req, &data, model, entity_names)
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
