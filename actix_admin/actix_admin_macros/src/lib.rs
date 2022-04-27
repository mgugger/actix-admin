use proc_macro;
use quote::quote;
use proc_macro2::{Span, Ident};
use syn::{ DeriveInput };

mod struct_fields;
use struct_fields::get_field_for_tokenstream;

#[proc_macro_derive(DeriveActixAdminModel)]
pub fn derive_crud_fns(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let fields = get_field_for_tokenstream(input);

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
                let db = &data.get_db();
                let entities = Entity::list_db(db, 1, 5);
                let entity_names = &data.get_actix_admin().entity_names;
                let model = ActixAdminViewModel::from(Entity);
                actix_admin::list_model(req, &data, model, entity_names)
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

            fn get_fields() -> Vec<(&'static str, ActixAdminField)> {
                let mut vec = Vec::new();
                let field_names = stringify!( 
                        #(#names_const_fields_str),*
                    ).split(",")
                    .collect::<Vec<_>>()
                    .into_iter()
                    .for_each( |field_name| 
                        vec.push((
                            field_name,
                            // TODO: derive correct AxtixAdminField Value
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
