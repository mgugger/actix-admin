use proc_macro;
use quote::quote;

#[proc_macro_derive(DeriveActixAdminModel)]
pub fn derive_crud_fns(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expanded = quote! {
        use std::convert::From;
        use async_trait::async_trait;
        use actix_admin::{ ActixAdminModelTrait, ActixAdminModel };

        impl From<Entity> for ActixAdminModel {
            fn from(entity: Entity) -> Self {
                ActixAdminModel {
                    fields: Vec::new()
                }
            }
        }
        #[async_trait]
        impl ActixAdminModelTrait for Entity {
            async fn list(db: &DatabaseConnection, page: usize, posts_per_page: usize) -> Vec<ActixAdminModel> {
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
                ActixAdminModel {
                    fields: Vec::new()
                }
                ]
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
