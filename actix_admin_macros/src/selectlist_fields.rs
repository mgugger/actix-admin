use syn::{
    DeriveInput, Ident
};
use quote::quote;
use crate::{model_fields::ModelField, struct_fields::{get_fields_for_tokenstream, get_tenant_ref_field}};
use proc_macro2::Span;

pub fn get_select_list_from_model(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let fields = get_fields_for_tokenstream(input);
    let tenant_ref_field = get_tenant_ref_field(&fields, false);

    let expanded = quote! {
        #[async_trait]
        impl ActixAdminSelectListTrait for Entity {
            async fn get_key_value(db: &DatabaseConnection, tenant_ref: Option<i32>) -> Result<Vec<(String, String)>, ActixAdminError> {
                let mut query = Entity::find().order_by_asc(Column::Id);
                #tenant_ref_field
                
                let entities = query.all(db).await?;

                let mut key_value = Vec::new();
            
                for entity in entities {
                    key_value.push((entity.id.to_string(),  entity.to_string()));
                };
                key_value.sort_by(|a, b| a.1.cmp(&b.1));
                Ok(key_value)
            }
        }
    };
    
    proc_macro::TokenStream::from(expanded)
}

pub fn get_select_list_from_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (_vis, ty, _generics) = (&ast.vis, &ast.ident, &ast.generics);

    let expanded = quote! {
        #[async_trait]
        impl ActixAdminSelectListTrait for #ty {
            async fn get_key_value(db: &DatabaseConnection, _tenant_ref: Option<i32>) -> Result<Vec<(String, String)>, ActixAdminError> {
                let mut fields = Vec::new();
                for field in #ty::iter() {
                    let field_val = field.to_string().trim_start_matches("'").trim_end_matches("'").to_string();
                    fields.push((field_val.clone(), field_val));
                }
                Ok(fields)
            }
        }
    };
    
    proc_macro::TokenStream::from(expanded)
}

pub fn get_select_lists(fields: &Vec<ModelField>) -> Vec<proc_macro2::TokenStream> {
    fields
    .iter()
    .filter(|model_field| model_field.select_list != "")
    .map(|model_field| {
        let ident_name = model_field.ident.to_string();
        let select_list_ident = Ident::new(&(model_field.select_list), Span::call_site());
        quote! {
            #ident_name => #select_list_ident::get_key_value(db, tenant_ref).await?
        }
    })
    .collect::<Vec<_>>()
}