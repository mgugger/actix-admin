use syn::{
    DeriveInput, Ident
};
use quote::quote;
use crate::model_fields::{ ModelField };
use proc_macro2::{Span};

pub fn get_select_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (_vis, ty, _generics) = (&ast.vis, &ast.ident, &ast.generics);

    let expanded = quote! {
        impl ActixAdminSelectListTrait for #ty {
            fn get_key_value() -> Vec<(String, String)> {
                let mut fields = Vec::new();
                for field in #ty::iter() {
                    fields.push((field.to_string(), field.to_string()));
                }
                fields
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
            #ident_name => #select_list_ident::get_key_value()
        }
    })
    .collect::<Vec<_>>()
}